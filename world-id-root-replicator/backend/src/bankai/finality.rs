use crate::config::BankaiNetwork;
use anyhow::{Context, Result};
use async_trait::async_trait;
use bankai_sdk::Bankai;
use bankai_types::api::ethereum::BankaiBlockFilterDto;

#[async_trait]
pub trait FinalityClient: Send + Sync {
    async fn finalized_execution_height(&self) -> Result<u64>;
}

pub struct BankaiFinalityClient {
    bankai: Bankai,
}

impl BankaiFinalityClient {
    pub fn new(network: BankaiNetwork, execution_rpc: String) -> Self {
        Self {
            bankai: Bankai::new(network.into_sdk(), Some(execution_rpc), None, None),
        }
    }
}

#[async_trait]
impl FinalityClient for BankaiFinalityClient {
    async fn finalized_execution_height(&self) -> Result<u64> {
        let height = self
            .bankai
            .api
            .ethereum()
            .execution()
            .height(&BankaiBlockFilterDto::finalized())
            .await
            .context("fetch Bankai finalized execution height")?;

        Ok(height.height)
    }
}
