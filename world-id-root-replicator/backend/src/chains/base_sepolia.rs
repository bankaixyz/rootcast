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

pub enum SubmissionCheck {
    Pending,
    Confirmed,
    Failed(String),
}

#[async_trait]
pub trait SubmissionClient: Send + Sync {
    async fn submit_artifact(
        &self,
        registry_address: Address,
        artifact_path: &str,
    ) -> Result<String>;
    async fn check_submission(&self, tx_hash: &str) -> Result<SubmissionCheck>;
}

pub struct BaseSepoliaSubmitter {
    rpc_url: String,
    private_key: String,
    chain_id: u64,
}

impl BaseSepoliaSubmitter {
    pub fn new(rpc_url: String, private_key: String, chain_id: u64) -> Self {
        Self {
            rpc_url,
            private_key,
            chain_id,
        }
    }

    async fn provider(&self) -> Result<impl Provider> {
        let signer: PrivateKeySigner = self
            .private_key
            .parse()
            .context("parse Base Sepolia private key")?;
        let wallet = EthereumWallet::from(signer);

        ProviderBuilder::new()
            .with_chain_id(self.chain_id)
            .wallet(wallet)
            .connect(self.rpc_url.as_str())
            .await
            .context("connect Base Sepolia provider")
    }
}

#[async_trait]
impl SubmissionClient for BaseSepoliaSubmitter {
    async fn submit_artifact(
        &self,
        registry_address: Address,
        artifact_path: &str,
    ) -> Result<String> {
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
            .context("send Base Sepolia submitRoot transaction")?;

        Ok(pending.tx_hash().to_string())
    }

    async fn check_submission(&self, tx_hash: &str) -> Result<SubmissionCheck> {
        let provider = self.provider().await?;
        let receipt = provider
            .get_transaction_receipt(tx_hash.parse()?)
            .await
            .context("fetch Base Sepolia transaction receipt")?;

        let Some(receipt) = receipt else {
            return Ok(SubmissionCheck::Pending);
        };

        if receipt.status() {
            Ok(SubmissionCheck::Confirmed)
        } else {
            Ok(SubmissionCheck::Failed(format!(
                "Base Sepolia transaction {tx_hash} reverted"
            )))
        }
    }
}
