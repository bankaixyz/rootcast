use crate::config::DestinationChainConfig;
use crate::db;
use crate::jobs::types::ObservedRoot;
use crate::world_id::SEPOLIA_IDENTITY_MANAGER;
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
        destinations: &[DestinationChainConfig],
    ) -> Result<()>;
}

pub struct WorldIdWatcher {
    rpc_url: String,
}

impl WorldIdWatcher {
    pub fn new(rpc_url: String) -> Self {
        Self { rpc_url }
    }
}

#[async_trait]
impl RootWatcher for WorldIdWatcher {
    async fn poll_once(
        &self,
        pool: &SqlitePool,
        destinations: &[DestinationChainConfig],
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
            .address(SEPOLIA_IDENTITY_MANAGER)
            .event_signature(TreeChanged::SIGNATURE_HASH)
            .from_block(from_block)
            .to_block(latest_block);

        let logs = provider
            .get_logs(&filter)
            .await
            .context("fetch TreeChanged logs")?;
        if logs.is_empty() {
            return Ok(());
        }

        if logs.len() > 1 {
            info!(
                from_block,
                latest_block,
                skipped_root_changes = logs.len() - 1,
                "coalescing World ID root changes to the newest event"
            );
        }

        match observed_root_from_log(logs.last().expect("logs is not empty")) {
            Ok(observed_root) => {
                let record = db::record_observed_root(pool, &observed_root, destinations).await?;
                if record.created {
                    info!(
                        root = %observed_root.root_hex,
                        source_block_number = observed_root.source_block_number,
                        source_tx_hash = %observed_root.source_tx_hash,
                        replaced_pending_roots = record.replaced_pending_count,
                        "detected latest World ID root"
                    );
                }
            }
            Err(error) => {
                warn!(?error, "skipping malformed World ID root change log");
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
