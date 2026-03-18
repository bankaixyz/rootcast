mod evm;
pub mod solana;
mod starknet;

pub use evm::EvmSubmitter;
pub use solana::SolanaSubmitter;
pub use starknet::StarknetSubmitter;

use anyhow::Result;
use async_trait::async_trait;

pub enum SubmissionCheck {
    Pending,
    Confirmed,
    Failed(String),
}

#[async_trait]
pub trait SubmissionClient: Send + Sync {
    async fn submit_artifact(&self, contract_address: &str, artifact_path: &str) -> Result<String>;
    async fn check_submission(&self, tx_hash: &str) -> Result<SubmissionCheck>;
}
