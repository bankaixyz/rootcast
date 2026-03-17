use crate::chains::{SubmissionCheck, SubmissionClient};
use crate::config::DestinationChainConfig;
use crate::proving::sp1::{starknet_proof_calldata, ProgramVkey};
use anyhow::{Context, Result};
use async_trait::async_trait;
use starknet::{
    accounts::{Account, ConnectedAccount, ExecutionEncoding, SingleOwnerAccount},
    core::{
        types::{BlockId, BlockTag, Call, ExecutionResult, Felt, StarknetError},
        utils::get_selector_from_name,
    },
    providers::{jsonrpc::HttpTransport, JsonRpcClient, Provider, ProviderError, Url},
    signers::{LocalWallet, SigningKey},
};

pub struct StarknetSubmitter {
    destination: DestinationChainConfig,
    program_vkey: ProgramVkey,
}

impl StarknetSubmitter {
    pub fn new(destination: DestinationChainConfig, program_vkey: ProgramVkey) -> Self {
        Self {
            destination,
            program_vkey,
        }
    }

    async fn create_account(
        &self,
    ) -> Result<SingleOwnerAccount<JsonRpcClient<HttpTransport>, LocalWallet>> {
        let provider = JsonRpcClient::new(HttpTransport::new(
            Url::parse(&self.destination.rpc_url).context("parse Starknet RPC URL")?,
        ));

        let private_key_felt =
            Felt::from_hex(&self.destination.private_key).context("parse Starknet private key")?;
        let signer = LocalWallet::from(SigningKey::from_secret_scalar(private_key_felt));

        let account_address = self
            .destination
            .account_address
            .as_deref()
            .context("missing Starknet account address in config")?;
        let address = Felt::from_hex(account_address).context("parse Starknet account address")?;
        let chain_id = provider
            .chain_id()
            .await
            .context("fetch Starknet chain id")?;

        let mut account =
            SingleOwnerAccount::new(provider, signer, address, chain_id, ExecutionEncoding::New);
        account.set_block_id(BlockId::Tag(BlockTag::Latest));
        Ok(account)
    }

    fn contract_address(&self, contract_address: &str) -> Result<Felt> {
        Felt::from_hex(contract_address).context("parse Starknet contract address")
    }

    fn transaction_hash_hex(&self, tx_hash: Felt) -> String {
        format!("{:#064x}", tx_hash)
    }
}

#[async_trait]
impl SubmissionClient for StarknetSubmitter {
    async fn submit_artifact(&self, contract_address: &str, artifact_path: &str) -> Result<String> {
        let account = self.create_account().await?;
        let call = Call {
            to: self.contract_address(contract_address)?,
            selector: get_selector_from_name("submit_root")
                .context("resolve Starknet submit_root selector")?,
            calldata: starknet_proof_calldata(artifact_path, &self.program_vkey)?,
        };

        let result = account
            .execute_v3(vec![call])
            .send()
            .await
            .context("send Starknet submit_root transaction")?;

        Ok(self.transaction_hash_hex(result.transaction_hash))
    }

    async fn check_submission(&self, tx_hash: &str) -> Result<SubmissionCheck> {
        let account = self.create_account().await?;
        let tx_hash_felt = Felt::from_hex(tx_hash).context("parse Starknet tx hash")?;
        let receipt = match account
            .provider()
            .get_transaction_receipt(tx_hash_felt)
            .await
        {
            Ok(receipt) => receipt,
            Err(ProviderError::StarknetError(StarknetError::TransactionHashNotFound)) => {
                return Ok(SubmissionCheck::Pending);
            }
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("fetch Starknet transaction receipt {tx_hash}"));
            }
        };

        match receipt.receipt.execution_result() {
            ExecutionResult::Succeeded => Ok(SubmissionCheck::Confirmed),
            ExecutionResult::Reverted { reason } => Ok(SubmissionCheck::Failed(format!(
                "starknet transaction {tx_hash} reverted: {reason}"
            ))),
        }
    }
}
