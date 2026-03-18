use crate::chains::{SubmissionCheck, SubmissionClient};
use crate::config::DestinationChainConfig;
use crate::proving::sp1::load_proof;
use alloy_network::EthereumWallet;
use alloy_primitives::Address;
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::{TransactionInput, TransactionRequest};
use alloy_signer_local::PrivateKeySigner;
use alloy_sol_types::{sol, SolCall};
use anyhow::{Context, Result};
use async_trait::async_trait;

sol! {
    function submitRoot(bytes proofBytes, bytes publicValues) external;
}

pub struct EvmSubmitter {
    destination: DestinationChainConfig,
}

impl EvmSubmitter {
    pub fn new(destination: DestinationChainConfig) -> Self {
        Self { destination }
    }

    async fn provider(&self) -> Result<impl Provider> {
        let signer: PrivateKeySigner = self
            .destination
            .private_key
            .parse()
            .with_context(|| format!("parse {} private key", self.destination.name()))?;
        let wallet = EthereumWallet::from(signer);

        ProviderBuilder::new()
            .with_chain_id(
                self.destination
                    .chain
                    .evm_chain_id()
                    .context("missing EVM chain id for destination")?,
            )
            .wallet(wallet)
            .connect(self.destination.rpc_url.as_str())
            .await
            .with_context(|| format!("connect {} provider", self.destination.name()))
    }
}

#[async_trait]
impl SubmissionClient for EvmSubmitter {
    async fn submit_artifact(&self, contract_address: &str, artifact_path: &str) -> Result<String> {
        let registry_address: Address = contract_address
            .parse()
            .with_context(|| format!("parse {} registry address", self.destination.name()))?;
        let proof = load_proof(artifact_path)?;
        let call = submitRootCall {
            proofBytes: proof.bytes().into(),
            publicValues: proof.public_values.to_vec().into(),
        };
        let provider = self.provider().await?;
        let transaction = TransactionRequest::default()
            .to(registry_address)
            .input(TransactionInput::both(call.abi_encode().into()));

        let pending = provider
            .send_transaction(transaction)
            .await
            .with_context(|| format!("send {} submitRoot transaction", self.destination.name()))?;

        Ok(pending.tx_hash().to_string())
    }

    async fn check_submission(&self, tx_hash: &str) -> Result<SubmissionCheck> {
        let provider = self.provider().await?;
        let receipt = provider
            .get_transaction_receipt(tx_hash.parse()?)
            .await
            .with_context(|| format!("fetch {} transaction receipt", self.destination.name()))?;

        let Some(receipt) = receipt else {
            return Ok(SubmissionCheck::Pending);
        };

        if receipt.status() {
            Ok(SubmissionCheck::Confirmed)
        } else {
            Ok(SubmissionCheck::Failed(format!(
                "{} transaction {tx_hash} reverted",
                self.destination.name()
            )))
        }
    }
}
