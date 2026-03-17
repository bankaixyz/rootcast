use crate::config::DestinationChainConfig;
use crate::db;
use crate::jobs::types::ObservedRoot;
use alloy_primitives::B256;
use alloy_provider::{Provider, ProviderBuilder};
use alloy_rpc_types_eth::{Filter, Log};
use alloy_sol_types::{sol, SolEvent};
use anyhow::{Context, Result};
use async_trait::async_trait;
use sqlx::SqlitePool;
use tracing::{info, warn};

const DEFAULT_EVENT_LOOKBACK_BLOCKS: u64 = 2_048;

sol! {
    event TreeChanged(uint256 indexed preRoot, uint8 indexed kind, uint256 indexed postRoot);
}

#[async_trait]
pub trait RootWatcher: Send + Sync {
    async fn poll_once(
        &self,
        pool: &SqlitePool,
        destination: &DestinationChainConfig,
    ) -> Result<()>;
}

pub struct WorldIdWatcher {
    rpc_url: String,
    identity_manager: alloy_primitives::Address,
}

impl WorldIdWatcher {
    pub fn new(rpc_url: String, identity_manager: alloy_primitives::Address) -> Self {
        Self {
            rpc_url,
            identity_manager,
        }
    }
}

#[async_trait]
impl RootWatcher for WorldIdWatcher {
    async fn poll_once(
        &self,
        pool: &SqlitePool,
        destination: &DestinationChainConfig,
    ) -> Result<()> {
        let provider = ProviderBuilder::new()
            .connect(self.rpc_url.as_str())
            .await
            .context("connect watcher provider")?;

        let latest_block = provider
            .get_block_number()
            .await
            .context("fetch latest execution block")?;

        let from_block = match db::latest_observed_source_block(pool).await? {
            Some(last_seen) => last_seen.saturating_add(1),
            None => latest_block.saturating_sub(DEFAULT_EVENT_LOOKBACK_BLOCKS),
        };

        if from_block > latest_block {
            return Ok(());
        }

        let filter = Filter::new()
            .address(self.identity_manager)
            .event_signature(TreeChanged::SIGNATURE_HASH)
            .from_block(from_block)
            .to_block(latest_block);

        let logs = provider
            .get_logs(&filter)
            .await
            .context("fetch TreeChanged logs")?;
        for log in logs {
            match observed_root_from_log(&log) {
                Ok(observed_root) => {
                    let created =
                        db::record_observed_root(pool, &observed_root, destination).await?;
                    if created {
                        info!(
                            root = %observed_root.root_hex,
                            source_block_number = observed_root.source_block_number,
                            source_tx_hash = %observed_root.source_tx_hash,
                            "detected new World ID root"
                        );
                    }
                }
                Err(error) => {
                    warn!(?error, "skipping malformed World ID root change log");
                }
            }
        }

        Ok(())
    }
}

fn observed_root_from_log(log: &Log) -> Result<ObservedRoot> {
    let decoded = log
        .log_decode_validate::<TreeChanged>()
        .context("decode TreeChanged log")?;

    let block_number = log
        .block_number
        .context("TreeChanged log missing block number")?;
    let tx_hash = log
        .transaction_hash
        .context("TreeChanged log missing transaction hash")?;

    let post_root = B256::from(decoded.data().postRoot.to_be_bytes());

    Ok(ObservedRoot {
        root_hex: post_root.to_string(),
        source_block_number: block_number,
        source_tx_hash: tx_hash.to_string(),
    })
}
