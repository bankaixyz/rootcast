use crate::config::BankaiNetwork;
use crate::world_id::{LATEST_ROOT_SLOT, SEPOLIA_IDENTITY_MANAGER};
use anyhow::{Context, Result};
use async_trait::async_trait;
use bankai_sdk::{Bankai, HashingFunction};
use bincode::serialize;

#[async_trait]
pub trait ProofBundleClient: Send + Sync {
    async fn fetch_exact_block_bundle(&self, source_block_number: u64) -> Result<Vec<u8>>;
}

pub struct BankaiProofBundleClient {
    bankai: Bankai,
}

impl BankaiProofBundleClient {
    pub fn new(network: BankaiNetwork, execution_rpc: String) -> Self {
        Self {
            bankai: Bankai::new(network.into_sdk(), Some(execution_rpc), None, None),
        }
    }
}

#[async_trait]
impl ProofBundleClient for BankaiProofBundleClient {
    async fn fetch_exact_block_bundle(&self, source_block_number: u64) -> Result<Vec<u8>> {
        let bundle = self
            .bankai
            .init_batch(None, HashingFunction::Keccak)
            .await
            .context("initialize Bankai proof batch")?
            .ethereum_storage_slot(
                source_block_number,
                SEPOLIA_IDENTITY_MANAGER,
                vec![LATEST_ROOT_SLOT],
            )
            .execute()
            .await
            .with_context(|| {
                format!("fetch storage-slot proof bundle for block {source_block_number}")
            })?;

        serialize(&bundle).context("serialize Bankai proof bundle")
    }
}
