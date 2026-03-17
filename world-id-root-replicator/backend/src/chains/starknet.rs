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
use tokio::time::{timeout, Duration};

const DEFAULT_STARKNET_L1_GAS: u64 = 50_000_000;
const DEFAULT_STARKNET_L1_GAS_PRICE_WEI: u128 = 1_000_000_000;
const STARKNET_CHAIN_ID_TIMEOUT: Duration = Duration::from_secs(15);
const STARKNET_SEND_TIMEOUT: Duration = Duration::from_secs(30);
const STARKNET_RECEIPT_TIMEOUT: Duration = Duration::from_secs(15);

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
        println!(
            "[starknet] creating account for {} via {}",
            self.destination.name(),
            self.destination.rpc_url
        );
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
        let chain_id = timeout(STARKNET_CHAIN_ID_TIMEOUT, provider.chain_id())
            .await
            .context("timed out fetching Starknet chain id")?
            .context("fetch Starknet chain id")?;

        let mut account =
            SingleOwnerAccount::new(provider, signer, address, chain_id, ExecutionEncoding::New);
        account.set_block_id(BlockId::Tag(BlockTag::Latest));
        println!(
            "[starknet] account ready for {} with chain id {chain_id:#x}",
            self.destination.name()
        );
        Ok(account)
    }

    fn contract_address(&self, contract_address: &str) -> Result<Felt> {
        Felt::from_hex(contract_address).context("parse Starknet contract address")
    }

    fn transaction_hash_hex(&self, tx_hash: Felt) -> String {
        format!("{:#064x}", tx_hash)
    }

    fn submit_root_calldata(&self, artifact_path: &str) -> Result<Vec<Felt>> {
        println!(
            "[starknet] generating submit_root calldata from {}",
            artifact_path
        );
        let proof = starknet_proof_calldata(artifact_path, &self.program_vkey)?;
        let calldata = serialize_felt_array_argument(proof);
        println!(
            "[starknet] generated calldata with {} felts",
            calldata.len()
        );
        Ok(calldata)
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
            calldata: self.submit_root_calldata(artifact_path)?,
        };

        println!(
            "[starknet] broadcasting submit_root to {}",
            contract_address
        );
        let result = timeout(
            STARKNET_SEND_TIMEOUT,
            account
                .execute_v3(vec![call])
                .l1_gas(DEFAULT_STARKNET_L1_GAS)
                .l1_gas_price(DEFAULT_STARKNET_L1_GAS_PRICE_WEI)
                .send(),
        )
        .await
        .context("timed out sending Starknet submit_root transaction")?
        .context("send Starknet submit_root transaction")?;

        println!(
            "[starknet] broadcasted transaction {}",
            self.transaction_hash_hex(result.transaction_hash)
        );

        Ok(self.transaction_hash_hex(result.transaction_hash))
    }

    async fn check_submission(&self, tx_hash: &str) -> Result<SubmissionCheck> {
        let account = self.create_account().await?;
        let tx_hash_felt = Felt::from_hex(tx_hash).context("parse Starknet tx hash")?;
        println!("[starknet] checking receipt for {}", tx_hash);
        let receipt = match timeout(
            STARKNET_RECEIPT_TIMEOUT,
            account.provider().get_transaction_receipt(tx_hash_felt),
        )
        .await
        .context("timed out fetching Starknet transaction receipt")?
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

fn serialize_felt_array_argument(values: Vec<Felt>) -> Vec<Felt> {
    let mut calldata = Vec::with_capacity(values.len() + 1);
    calldata.push(Felt::from(values.len()));
    calldata.extend(values);
    calldata
}

#[cfg(test)]
mod tests {
    use super::serialize_felt_array_argument;
    use starknet::core::types::Felt;

    #[test]
    fn serializes_felt_array_argument_with_length_prefix() {
        let calldata = serialize_felt_array_argument(vec![Felt::ONE, Felt::TWO]);

        assert_eq!(calldata, vec![Felt::from(2_u64), Felt::ONE, Felt::TWO]);
    }
}
