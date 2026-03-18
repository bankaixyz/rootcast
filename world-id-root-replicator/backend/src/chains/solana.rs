use super::{SubmissionCheck, SubmissionClient};
use crate::config::DestinationChainConfig;
use crate::proving::sp1::{decode_public_values, load_proof};
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use solana_client::rpc_client::RpcClient;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::{AccountMeta, Instruction};
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::{Keypair, Signature, Signer};
use solana_sdk::transaction::Transaction;
use std::fs;
use std::str::FromStr;

const COMPUTE_UNIT_LIMIT: u32 = 1_400_000;
const ROOT_SEED: &[u8] = b"root";
const STATE_SEED: &[u8] = b"state";

pub struct SolanaSubmitter {
    destination: DestinationChainConfig,
    keypair: Keypair,
    rpc_client: RpcClient,
}

impl SolanaSubmitter {
    pub fn new(destination: DestinationChainConfig) -> Result<Self> {
        let keypair = load_solana_keypair(&destination.private_key)?;
        let rpc_client = RpcClient::new_with_commitment(
            destination.rpc_url.clone(),
            CommitmentConfig::confirmed(),
        );

        Ok(Self {
            destination,
            keypair,
            rpc_client,
        })
    }
}

#[async_trait]
impl SubmissionClient for SolanaSubmitter {
    async fn submit_artifact(&self, contract_address: &str, artifact_path: &str) -> Result<String> {
        let program_id = parse_pubkey(contract_address, self.destination.name())?;
        let proof = load_proof(artifact_path)?;
        let public_values = proof.public_values.to_vec();
        let decoded = decode_public_values(&public_values)?;
        let (state_pda, _) = state_pda(&program_id);
        let (root_record_pda, _) = root_record_pda(&program_id, decoded.source_block_number);

        let instruction = submit_root_instruction(
            program_id,
            self.keypair.pubkey(),
            state_pda,
            root_record_pda,
            decoded.source_block_number,
            public_values,
            proof.bytes(),
        )?;

        send_instruction(
            &self.rpc_client,
            &self.keypair,
            vec![
                ComputeBudgetInstruction::set_compute_unit_limit(COMPUTE_UNIT_LIMIT),
                instruction,
            ],
        )
    }

    async fn check_submission(&self, tx_hash: &str) -> Result<SubmissionCheck> {
        let signature: Signature = tx_hash
            .parse()
            .with_context(|| format!("parse {} transaction signature", self.destination.name()))?;
        let statuses = self
            .rpc_client
            .get_signature_statuses_with_history(&[signature])?;
        let Some(status) = statuses.value.into_iter().next().flatten() else {
            return Ok(SubmissionCheck::Pending);
        };

        if let Some(error) = status.err {
            return Ok(SubmissionCheck::Failed(format!(
                "{} transaction {tx_hash} failed: {error}",
                self.destination.name()
            )));
        }

        Ok(SubmissionCheck::Confirmed)
    }
}

pub fn initialize_registry(
    rpc_url: &str,
    private_key: &str,
    program_id: &str,
    program_vkey: &str,
) -> Result<String> {
    let keypair = load_solana_keypair(private_key)?;
    let rpc_client =
        RpcClient::new_with_commitment(rpc_url.to_string(), CommitmentConfig::confirmed());
    let program_id = parse_pubkey(program_id, "solana-devnet")?;
    let (state_pda, _) = state_pda(&program_id);
    let program_vkey_hash = parse_program_vkey(program_vkey)?;

    let instruction =
        initialize_instruction(program_id, keypair.pubkey(), state_pda, program_vkey_hash)?;

    send_instruction(&rpc_client, &keypair, vec![instruction])
}

pub fn state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[STATE_SEED], program_id)
}

pub fn root_record_pda(program_id: &Pubkey, source_block_number: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[ROOT_SEED, &source_block_number.to_be_bytes()], program_id)
}

pub fn load_solana_keypair(private_key_str: &str) -> Result<Keypair> {
    if private_key_str.trim().starts_with('[') {
        let bytes: Vec<u8> =
            serde_json::from_str(private_key_str).context("parse JSON-array Solana private key")?;
        return keypair_from_bytes(bytes);
    }

    if let Ok(bytes) = bs58::decode(private_key_str).into_vec() {
        if bytes.len() == 64 {
            return keypair_from_bytes(bytes);
        }
    }

    if let Ok(contents) = fs::read_to_string(private_key_str) {
        let bytes: Vec<u8> =
            serde_json::from_str(&contents).context("parse Solana keypair file")?;
        return keypair_from_bytes(bytes);
    }

    Err(anyhow!(
        "invalid Solana private key format; expected JSON array, base58 string, or file path"
    ))
}

fn keypair_from_bytes(bytes: Vec<u8>) -> Result<Keypair> {
    if bytes.len() != 64 {
        anyhow::bail!("Solana keypair must be 64 bytes, got {}", bytes.len());
    }

    Keypair::try_from(bytes.as_slice()).context("construct Solana keypair")
}

fn parse_program_vkey(program_vkey: &str) -> Result<[u8; 32]> {
    let bytes = hex::decode(program_vkey.trim_start_matches("0x"))
        .context("decode PROGRAM_VKEY as hex bytes")?;
    if bytes.len() != 32 {
        anyhow::bail!("PROGRAM_VKEY must be 32 bytes, got {}", bytes.len());
    }

    let mut program_vkey_hash = [0u8; 32];
    program_vkey_hash.copy_from_slice(&bytes);
    Ok(program_vkey_hash)
}

fn initialize_instruction(
    program_id: Pubkey,
    payer: Pubkey,
    state_pda: Pubkey,
    program_vkey_hash: [u8; 32],
) -> Result<Instruction> {
    let mut data = Vec::new();
    data.extend_from_slice(&discriminator("initialize"));
    data.extend_from_slice(&borsh::to_vec(&program_vkey_hash)?);

    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    })
}

fn submit_root_instruction(
    program_id: Pubkey,
    payer: Pubkey,
    state_pda: Pubkey,
    root_record_pda: Pubkey,
    source_block_number: u64,
    public_values: Vec<u8>,
    groth16_proof: Vec<u8>,
) -> Result<Instruction> {
    let mut data = Vec::new();
    data.extend_from_slice(&discriminator("submit_root"));
    data.extend_from_slice(&borsh::to_vec(&source_block_number)?);
    data.extend_from_slice(&borsh::to_vec(&public_values)?);
    data.extend_from_slice(&borsh::to_vec(&groth16_proof)?);

    Ok(Instruction {
        program_id,
        accounts: vec![
            AccountMeta::new(state_pda, false),
            AccountMeta::new(root_record_pda, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(system_program_id(), false),
        ],
        data,
    })
}

fn send_instruction(
    rpc_client: &RpcClient,
    keypair: &Keypair,
    instructions: Vec<Instruction>,
) -> Result<String> {
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    let transaction = Transaction::new_signed_with_payer(
        &instructions,
        Some(&keypair.pubkey()),
        &[keypair],
        recent_blockhash,
    );

    rpc_client
        .send_and_confirm_transaction(&transaction)
        .map(|signature| signature.to_string())
        .context("send and confirm Solana transaction")
}

fn parse_pubkey(contract_address: &str, chain_name: &str) -> Result<Pubkey> {
    Pubkey::from_str(contract_address)
        .with_context(|| format!("parse {chain_name} target address as Solana pubkey"))
}

fn system_program_id() -> Pubkey {
    Pubkey::from_str("11111111111111111111111111111111").expect("system program id is valid")
}

fn discriminator(name: &str) -> [u8; 8] {
    let preimage = format!("global:{name}");
    let mut discriminator = [0u8; 8];
    discriminator.copy_from_slice(&solana_sdk::hash::hash(preimage.as_bytes()).to_bytes()[..8]);
    discriminator
}
