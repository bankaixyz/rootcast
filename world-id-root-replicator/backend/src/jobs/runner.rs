use crate::bankai::finality::{BankaiFinalityClient, FinalityClient};
use crate::bankai::proof_bundle::{BankaiProofBundleClient, ProofBundleClient};
use crate::chains::base_sepolia::{BaseSepoliaSubmitter, SubmissionCheck, SubmissionClient};
use crate::config::Config;
use crate::db::{self, ActiveJob};
use crate::jobs::types::{ChainSubmissionState, ReplicationJobState};
use crate::proving::sp1::{root_hex_to_bytes, ProofService, PublicValues, Sp1ProofService};
use crate::world_id::watcher::{RootWatcher, WorldIdWatcher};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

const LOOP_INTERVAL: Duration = Duration::from_secs(5);
const MIN_PROOF_REQUEST_AGE_SECS: i64 = 50 * 60;

pub struct Runner {
    pool: SqlitePool,
    chain_name: &'static str,
    destination_registry_address: alloy_primitives::Address,
    destination: crate::config::DestinationChainConfig,
    watcher: Arc<dyn RootWatcher>,
    finality_client: Arc<dyn FinalityClient>,
    bundle_client: Arc<dyn ProofBundleClient>,
    proof_service: Arc<dyn ProofService>,
    submission_client: Arc<dyn SubmissionClient>,
}

impl Runner {
    pub fn from_config(config: Config, pool: SqlitePool) -> Result<Self> {
        let watcher = Arc::new(WorldIdWatcher::new(config.execution_rpc.clone()));
        let finality_client = Arc::new(BankaiFinalityClient::new(
            config.bankai_network,
            config.execution_rpc.clone(),
        ));
        let bundle_client = Arc::new(BankaiProofBundleClient::new(
            config.bankai_network,
            config.execution_rpc.clone(),
        ));
        let proof_service = Arc::new(Sp1ProofService::new(PathBuf::from("artifacts/proofs")));
        let submission_client = Arc::new(BaseSepoliaSubmitter::new(
            config.base_sepolia.rpc_url.clone(),
            config.base_sepolia.private_key.clone(),
            config.base_sepolia.chain_id,
        ));

        Ok(Self {
            pool,
            chain_name: config.base_sepolia.name,
            destination_registry_address: config.base_sepolia.registry_address,
            destination: config.base_sepolia,
            watcher,
            finality_client,
            bundle_client,
            proof_service,
            submission_client,
        })
    }

    #[cfg(test)]
    fn new_for_tests(
        pool: SqlitePool,
        destination: crate::config::DestinationChainConfig,
        watcher: Arc<dyn RootWatcher>,
        finality_client: Arc<dyn FinalityClient>,
        bundle_client: Arc<dyn ProofBundleClient>,
        proof_service: Arc<dyn ProofService>,
        submission_client: Arc<dyn SubmissionClient>,
    ) -> Self {
        Self {
            pool,
            chain_name: destination.name,
            destination_registry_address: destination.registry_address,
            destination,
            watcher,
            finality_client,
            bundle_client,
            proof_service,
            submission_client,
        }
    }

    pub async fn run_forever(self) -> Result<()> {
        loop {
            self.advance_once().await?;
            sleep(LOOP_INTERVAL).await;
        }
    }

    pub async fn advance_once(&self) -> Result<()> {
        self.watcher
            .poll_once(&self.pool, &self.destination)
            .await
            .context("poll World ID watcher")?;

        db::repair_inflight_jobs(&self.pool).await?;

        let Some(job) = db::next_active_job(&self.pool, self.chain_name).await? else {
            return Ok(());
        };

        self.advance_job(job).await
    }

    async fn advance_job(&self, job: ActiveJob) -> Result<()> {
        match job.job_state {
            ReplicationJobState::WaitingFinality => self.advance_waiting_finality(job).await,
            ReplicationJobState::ReadyToProve => self.advance_ready_to_prove(job).await,
            ReplicationJobState::ProofReady | ReplicationJobState::Submitting => {
                self.advance_submission(job).await
            }
            ReplicationJobState::ProofInProgress
            | ReplicationJobState::Completed
            | ReplicationJobState::Failed => Ok(()),
        }
    }

    async fn advance_waiting_finality(&self, job: ActiveJob) -> Result<()> {
        let finalized_height = match self.finality_client.finalized_execution_height().await {
            Ok(height) => height,
            Err(error) => {
                warn!(
                    ?error,
                    job_id = job.job_id,
                    source_block_number = job.source_block_number,
                    "failed to fetch Bankai finality"
                );
                db::mark_job_retryable(
                    &self.pool,
                    job.job_id,
                    ReplicationJobState::WaitingFinality,
                    &error.to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        let ready = finalized_height >= job.source_block_number;
        if finalized_height < job.source_block_number {
            info!(
                job_id = job.job_id,
                root = %job.root_hex,
                bankai_finalized_height = finalized_height,
                required_source_block = job.source_block_number,
                decision = if ready { "ready_to_prove" } else { "wait" },
                "checked Bankai finality"
            );
            return Ok(());
        }

        info!(
            job_id = job.job_id,
            root = %job.root_hex,
            bankai_finalized_height = finalized_height,
            required_source_block = job.source_block_number,
            decision = "ready_to_prove",
            "checked Bankai finality"
        );
        db::mark_observed_root_finalized(&self.pool, job.observed_root_id).await?;
        db::update_job_state(&self.pool, job.job_id, ReplicationJobState::ReadyToProve).await?;
        Ok(())
    }

    async fn advance_ready_to_prove(&self, job: ActiveJob) -> Result<()> {
        if job.proof_requested_at.is_none() {
            if let Some(last_request) = db::latest_proof_request(&self.pool).await? {
                if last_request.age_seconds < MIN_PROOF_REQUEST_AGE_SECS {
                    return Ok(());
                }
            }
        }

        let bundle_bytes = match self
            .bundle_client
            .fetch_exact_block_bundle(job.source_block_number)
            .await
        {
            Ok(bundle) => bundle,
            Err(error) => {
                db::mark_job_retryable(
                    &self.pool,
                    job.job_id,
                    ReplicationJobState::ReadyToProve,
                    &error.to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        db::mark_job_proof_in_progress(&self.pool, job.job_id).await?;

        let expected_public_values = PublicValues {
            source_block_number: job.source_block_number,
            root: root_hex_to_bytes(&job.root_hex)?,
        };

        let proof_artifact = match self
            .proof_service
            .prove(job.job_id, &bundle_bytes, &expected_public_values)
            .await
        {
            Ok(artifact) => artifact,
            Err(error) => {
                let next_state = if is_terminal_proving_error(&error) {
                    ReplicationJobState::Failed
                } else {
                    ReplicationJobState::ReadyToProve
                };

                if next_state == ReplicationJobState::Failed {
                    db::mark_job_failed(&self.pool, job.job_id, &error.to_string()).await?;
                } else {
                    db::mark_job_retryable(
                        &self.pool,
                        job.job_id,
                        ReplicationJobState::ReadyToProve,
                        &error.to_string(),
                    )
                    .await?;
                }

                return Ok(());
            }
        };

        db::mark_job_proof_ready(&self.pool, job.job_id, &proof_artifact.path).await?;
        Ok(())
    }

    async fn advance_submission(&self, job: ActiveJob) -> Result<()> {
        match job.submission_state {
            ChainSubmissionState::Pending => {
                let Some(artifact_path) = job.proof_artifact_ref.as_deref() else {
                    db::mark_job_retryable(
                        &self.pool,
                        job.job_id,
                        ReplicationJobState::ReadyToProve,
                        "missing proof artifact ref for submission",
                    )
                    .await?;
                    return Ok(());
                };

                let proof_artifact = self.proof_service.load(artifact_path).await?;
                if proof_artifact.decoded_public_values.source_block_number
                    != job.source_block_number
                    || crate::proving::sp1::root_to_hex(proof_artifact.decoded_public_values.root)
                        != job.root_hex
                {
                    db::mark_job_failed(
                        &self.pool,
                        job.job_id,
                        "proof artifact public values do not match the observed root",
                    )
                    .await?;
                    db::mark_submission_failed(
                        &self.pool,
                        job.submission_id,
                        "proof artifact public values do not match the observed root",
                    )
                    .await?;
                    return Ok(());
                }

                let tx_hash = match self
                    .submission_client
                    .submit_artifact(self.destination_registry_address, artifact_path)
                    .await
                {
                    Ok(tx_hash) => tx_hash,
                    Err(error) => {
                        db::mark_job_retryable(
                            &self.pool,
                            job.job_id,
                            ReplicationJobState::ProofReady,
                            &error.to_string(),
                        )
                        .await?;
                        db::mark_submission_retryable(
                            &self.pool,
                            job.submission_id,
                            ChainSubmissionState::Pending,
                            &error.to_string(),
                        )
                        .await?;
                        return Ok(());
                    }
                };

                db::mark_submission_submitting(&self.pool, job.submission_id, &tx_hash).await?;
                db::update_job_state(&self.pool, job.job_id, ReplicationJobState::Submitting)
                    .await?;
                info!(job_id = job.job_id, %tx_hash, "submitted Base Sepolia proof");
                Ok(())
            }
            ChainSubmissionState::Submitting => {
                let Some(tx_hash) = job.submission_tx_hash.as_deref() else {
                    db::mark_job_retryable(
                        &self.pool,
                        job.job_id,
                        ReplicationJobState::ProofReady,
                        "submission entered submitting state without a transaction hash",
                    )
                    .await?;
                    db::mark_submission_retryable(
                        &self.pool,
                        job.submission_id,
                        ChainSubmissionState::Pending,
                        "submission entered submitting state without a transaction hash",
                    )
                    .await?;
                    return Ok(());
                };

                match self.submission_client.check_submission(tx_hash).await {
                    Ok(SubmissionCheck::Pending) => Ok(()),
                    Ok(SubmissionCheck::Confirmed) => {
                        db::mark_submission_confirmed(&self.pool, job.submission_id, tx_hash)
                            .await?;
                        db::mark_job_completed(&self.pool, job.job_id).await?;
                        Ok(())
                    }
                    Ok(SubmissionCheck::Failed(message)) => {
                        db::mark_submission_failed(&self.pool, job.submission_id, &message).await?;
                        db::mark_job_failed(&self.pool, job.job_id, &message).await?;
                        Ok(())
                    }
                    Err(error) => {
                        warn!(?error, job_id = job.job_id, "failed to confirm submission");
                        Ok(())
                    }
                }
            }
            ChainSubmissionState::Confirmed | ChainSubmissionState::Failed => Ok(()),
        }
    }
}

fn is_terminal_proving_error(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .contains("decoded proof public values do not match observed root")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DestinationChainConfig;
    use crate::jobs::types::ObservedRoot;
    use alloy_primitives::Address;
    use anyhow::Result;
    use async_trait::async_trait;
    use sqlx::SqlitePool;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    struct NoopWatcher;

    #[async_trait]
    impl RootWatcher for NoopWatcher {
        async fn poll_once(
            &self,
            _pool: &SqlitePool,
            _destination: &DestinationChainConfig,
        ) -> Result<()> {
            Ok(())
        }
    }

    struct StaticFinalityClient {
        height: u64,
    }

    #[async_trait]
    impl FinalityClient for StaticFinalityClient {
        async fn finalized_execution_height(&self) -> Result<u64> {
            Ok(self.height)
        }
    }

    struct FailingFinalityClient;

    #[async_trait]
    impl FinalityClient for FailingFinalityClient {
        async fn finalized_execution_height(&self) -> Result<u64> {
            anyhow::bail!("bankai finality unavailable")
        }
    }

    struct StaticBundleClient;

    #[async_trait]
    impl ProofBundleClient for StaticBundleClient {
        async fn fetch_exact_block_bundle(&self, _source_block_number: u64) -> Result<Vec<u8>> {
            Ok(vec![1, 2, 3, 4])
        }
    }

    struct FakeProofService {
        prove_calls: AtomicU64,
    }

    #[async_trait]
    impl ProofService for FakeProofService {
        async fn prove(
            &self,
            job_id: i64,
            _bundle_bytes: &[u8],
            expected_public_values: &PublicValues,
        ) -> Result<crate::proving::sp1::ProofArtifact> {
            self.prove_calls.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!("fake-proof-{job_id}.bin"));
            std::fs::write(&path, b"proof").unwrap();
            Ok(crate::proving::sp1::ProofArtifact {
                path: path.display().to_string(),
                public_values: vec![],
                decoded_public_values: expected_public_values.clone(),
            })
        }

        async fn load(&self, artifact_path: &str) -> Result<crate::proving::sp1::ProofArtifact> {
            Ok(crate::proving::sp1::ProofArtifact {
                path: artifact_path.to_string(),
                public_values: vec![],
                decoded_public_values: PublicValues {
                    source_block_number: 12_345,
                    root: [7u8; 32],
                },
            })
        }
    }

    struct FakeSubmissionClient {
        submitted: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl SubmissionClient for FakeSubmissionClient {
        async fn submit_artifact(
            &self,
            _registry_address: Address,
            artifact_path: &str,
        ) -> Result<String> {
            self.submitted
                .lock()
                .unwrap()
                .push(artifact_path.to_string());
            Ok("0x1111111111111111111111111111111111111111111111111111111111111111".to_string())
        }

        async fn check_submission(&self, _tx_hash: &str) -> Result<SubmissionCheck> {
            Ok(SubmissionCheck::Confirmed)
        }
    }

    #[tokio::test]
    async fn runner_waits_for_bankai_finality() {
        let pool = test_pool().await;
        let destination = destination();
        db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([5u8; 32])),
                source_block_number: 100,
                source_tx_hash: "0xabc".to_string(),
            },
            &destination,
        )
        .await
        .unwrap();

        let runner = Runner::new_for_tests(
            pool.clone(),
            destination,
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 99 }),
            Arc::new(StaticBundleClient),
            Arc::new(FakeProofService {
                prove_calls: AtomicU64::new(0),
            }),
            Arc::new(FakeSubmissionClient {
                submitted: Mutex::new(Vec::new()),
            }),
        );

        runner.advance_once().await.unwrap();

        let job = db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.job_state, ReplicationJobState::WaitingFinality);
    }

    #[tokio::test]
    async fn recording_the_same_root_twice_reuses_the_same_job() {
        let pool = test_pool().await;
        let destination = destination();
        let observed_root = ObservedRoot {
            root_hex: format!("0x{}", hex::encode([9u8; 32])),
            source_block_number: 77,
            source_tx_hash: "0xaaa".to_string(),
        };

        let created = db::record_observed_root(&pool, &observed_root, &destination)
            .await
            .unwrap();
        let created_again = db::record_observed_root(&pool, &observed_root, &destination)
            .await
            .unwrap();

        let observed_root_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM observed_roots")
                .fetch_one(&pool)
                .await
                .unwrap();
        let job_count = sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM replication_jobs")
            .fetch_one(&pool)
            .await
            .unwrap();

        assert!(created.created);
        assert!(!created_again.created);
        assert_eq!(observed_root_count, 1);
        assert_eq!(job_count, 1);
    }

    #[tokio::test]
    async fn recording_a_new_root_replaces_older_pending_roots() {
        let pool = test_pool().await;
        let destination = destination();

        db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([1u8; 32])),
                source_block_number: 10,
                source_tx_hash: "0x111".to_string(),
            },
            &destination,
        )
        .await
        .unwrap();

        let created = db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([2u8; 32])),
                source_block_number: 20,
                source_tx_hash: "0x222".to_string(),
            },
            &destination,
        )
        .await
        .unwrap();

        let roots = sqlx::query_as::<_, (String, i64)>(
            "SELECT root_hex, source_block_number FROM observed_roots",
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert!(created.created);
        assert_eq!(created.replaced_pending_count, 1);
        assert_eq!(roots.len(), 1);
        assert_eq!(roots[0].1, 20);
    }

    #[tokio::test]
    async fn runner_persists_retryable_failures() {
        let pool = test_pool().await;
        let destination = destination();
        db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([3u8; 32])),
                source_block_number: 55,
                source_tx_hash: "0xbbb".to_string(),
            },
            &destination,
        )
        .await
        .unwrap();

        let runner = Runner::new_for_tests(
            pool.clone(),
            destination,
            Arc::new(NoopWatcher),
            Arc::new(FailingFinalityClient),
            Arc::new(StaticBundleClient),
            Arc::new(FakeProofService {
                prove_calls: AtomicU64::new(0),
            }),
            Arc::new(FakeSubmissionClient {
                submitted: Mutex::new(Vec::new()),
            }),
        );

        runner.advance_once().await.unwrap();

        let job = db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.job_state, ReplicationJobState::WaitingFinality);
        assert_eq!(job.job_retry_count, 1);
        assert_eq!(
            job.job_error_message.as_deref(),
            Some("bankai finality unavailable")
        );
    }

    #[tokio::test]
    async fn runner_proves_and_submits_without_reproving_after_restart() {
        let pool = test_pool().await;
        let destination = destination();
        db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([7u8; 32])),
                source_block_number: 12_345,
                source_tx_hash: "0xdef".to_string(),
            },
            &destination,
        )
        .await
        .unwrap();

        let submission_client = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
        });
        let proof_service = Arc::new(FakeProofService {
            prove_calls: AtomicU64::new(0),
        });

        let runner = Runner::new_for_tests(
            pool.clone(),
            destination,
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            proof_service.clone(),
            submission_client.clone(),
        );

        runner.advance_once().await.unwrap();
        let job = db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.job_state, ReplicationJobState::ReadyToProve);

        runner.advance_once().await.unwrap();
        let job = db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.job_state, ReplicationJobState::ProofReady);

        runner.advance_once().await.unwrap();
        let job = db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.job_state, ReplicationJobState::Submitting);
        assert_eq!(submission_client.submitted.lock().unwrap().len(), 1);

        runner.advance_once().await.unwrap();
        assert!(db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .is_none());
        assert_eq!(submission_client.submitted.lock().unwrap().len(), 1);
        assert_eq!(proof_service.prove_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn runner_rate_limits_new_proof_requests() {
        let pool = test_pool().await;
        let destination = destination();
        let proof_service = Arc::new(FakeProofService {
            prove_calls: AtomicU64::new(0),
        });

        db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([3u8; 32])),
                source_block_number: 100,
                source_tx_hash: "0xold".to_string(),
            },
            &destination,
        )
        .await
        .unwrap();
        sqlx::query(
            r#"
            UPDATE replication_jobs
            SET
                state = ?,
                proof_requested_at = datetime('now', '-49 minutes')
            "#,
        )
        .bind(ReplicationJobState::Completed.as_db_str())
        .execute(&pool)
        .await
        .unwrap();

        db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([7u8; 32])),
                source_block_number: 12_345,
                source_tx_hash: "0xnew".to_string(),
            },
            &destination,
        )
        .await
        .unwrap();

        let runner = Runner::new_for_tests(
            pool.clone(),
            destination,
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            proof_service.clone(),
            Arc::new(FakeSubmissionClient {
                submitted: Mutex::new(Vec::new()),
            }),
        );

        runner.advance_once().await.unwrap();
        let job = db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.job_state, ReplicationJobState::ReadyToProve);

        runner.advance_once().await.unwrap();
        let job = db::next_active_job(&pool, "base-sepolia")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(job.job_state, ReplicationJobState::ReadyToProve);
        assert!(job.proof_requested_at.is_none());
        assert_eq!(proof_service.prove_calls.load(Ordering::Relaxed), 0);
    }

    async fn test_pool() -> SqlitePool {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
        let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path =
            std::env::temp_dir().join(format!("world-id-root-replicator-{unique}-{counter}.db"));
        let database_url = format!("sqlite://{}", path.display());
        let pool = crate::db::connect(&database_url).await.unwrap();
        crate::db::migrate(&pool).await.unwrap();
        pool
    }

    fn destination() -> DestinationChainConfig {
        DestinationChainConfig {
            name: "base-sepolia",
            chain_id: 84_532,
            rpc_url: "https://example.invalid".to_string(),
            registry_address: "0x0000000000000000000000000000000000000123"
                .parse()
                .unwrap(),
            private_key: "0x01".to_string(),
        }
    }
}
