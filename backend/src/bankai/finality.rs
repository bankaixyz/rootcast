use crate::config::BankaiNetwork;
use anyhow::{Context, Result};
use async_trait::async_trait;
use bankai_sdk::Bankai;
use bankai_types::api::ethereum::BankaiBlockFilterDto;
use tokio::time::{timeout, Duration};

const BANKAI_FINALITY_TIMEOUT: Duration = Duration::from_secs(30);

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
        let height = timeout(
            BANKAI_FINALITY_TIMEOUT,
            self.bankai
                .api
                .ethereum()
                .execution()
                .height(&BankaiBlockFilterDto::finalized()),
        )
        .await
        .context("timed out fetching Bankai finalized execution height")?
        .context("fetch Bankai finalized execution height")?;

        Ok(height.height)
    }
}
