use crate::bankai::finality::{BankaiFinalityClient, FinalityClient};
use crate::bankai::proof_bundle::{BankaiProofBundleClient, ProofBundleClient};
use crate::chains::{
    EvmSubmitter, SolanaSubmitter, StarknetSubmitter, SubmissionCheck, SubmissionClient,
};
use crate::config::{Config, DestinationChainConfig};
use crate::db::{self, ActiveJob, ChainSubmission};
use crate::jobs::types::{ChainSubmissionState, DestinationChain, ReplicationJobState};
use crate::proving::sp1::{
    current_program_vkey, root_hex_to_bytes, root_to_hex, ProofService, PublicValues,
    Sp1ProofService,
};
use crate::world_id::watcher::{RootWatcher, WorldIdWatcher};
use anyhow::{Context, Result};
use sqlx::SqlitePool;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{info, warn};

const LOOP_INTERVAL: Duration = Duration::from_secs(5);
const MAX_JOB_RPC_FAILURES: u32 = 5;
const MAX_SUBMISSION_RPC_FAILURES: u32 = 5;

pub struct Runner {
    pool: SqlitePool,
    destinations: Vec<DestinationChainConfig>,
    submission_clients: HashMap<&'static str, Arc<dyn SubmissionClient>>,
    watcher: Arc<dyn RootWatcher>,
    finality_client: Arc<dyn FinalityClient>,
    bundle_client: Arc<dyn ProofBundleClient>,
    proof_service: Arc<dyn ProofService>,
}

impl Runner {
    pub fn from_config(config: Config, pool: SqlitePool) -> Result<Self> {
        let watcher = Arc::new(WorldIdWatcher::new(
            config.execution_rpc.clone(),
            config.enforce_min_proof_request_gap,
        ));
        let finality_client = Arc::new(BankaiFinalityClient::new(
            config.bankai_network,
            config.execution_rpc.clone(),
        ));
        let bundle_client = Arc::new(BankaiProofBundleClient::new(
            config.bankai_network,
            config.execution_rpc.clone(),
        ));
        let proof_service = Arc::new(Sp1ProofService::new(PathBuf::from("artifacts/proofs")));
        let destinations = config.destination_chains;
        let program_vkey = current_program_vkey();
        let submission_clients = destinations
            .iter()
            .cloned()
            .map(|destination| {
                let client: Arc<dyn SubmissionClient> = match destination.chain {
                    DestinationChain::StarknetSepolia => Arc::new(StarknetSubmitter::new(
                        destination.clone(),
                        program_vkey.clone(),
                    )),
                    DestinationChain::SolanaDevnet => {
                        Arc::new(SolanaSubmitter::new(destination.clone())?)
                    }
                    _ => Arc::new(EvmSubmitter::new(destination.clone())),
                };

                Ok((destination.name(), client))
            })
            .collect::<Result<HashMap<_, _>>>()?;

        Ok(Self {
            pool,
            destinations,
            submission_clients,
            watcher,
            finality_client,
            bundle_client,
            proof_service,
        })
    }

    #[cfg(test)]
    fn new_for_tests(
        pool: SqlitePool,
        destinations: Vec<DestinationChainConfig>,
        submission_clients: HashMap<&'static str, Arc<dyn SubmissionClient>>,
        watcher: Arc<dyn RootWatcher>,
        finality_client: Arc<dyn FinalityClient>,
        bundle_client: Arc<dyn ProofBundleClient>,
        proof_service: Arc<dyn ProofService>,
    ) -> Self {
        Self {
            pool,
            destinations,
            submission_clients,
            watcher,
            finality_client,
            bundle_client,
            proof_service,
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
            .poll_once(&self.pool, &self.destinations)
            .await
            .context("poll World ID watcher")?;

        db::repair_inflight_jobs(&self.pool).await?;

        let jobs = db::active_jobs(&self.pool).await?;
        if jobs.is_empty() {
            return Ok(());
        }

        for job in jobs {
            self.advance_job(job).await?;
        }

        Ok(())
    }

    async fn advance_job(&self, job: ActiveJob) -> Result<()> {
        match job.job_state {
            ReplicationJobState::WaitingFinality => self.advance_waiting_finality(job).await,
            ReplicationJobState::ReadyToProve => self.advance_ready_to_prove(job).await,
            ReplicationJobState::ProofReady | ReplicationJobState::Submitting => {
                self.advance_submissions(job).await
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
                self.mark_job_retryable_or_failed(
                    &job,
                    ReplicationJobState::WaitingFinality,
                    &error.to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        if finalized_height < job.source_block_number {
            info!(
                job_id = job.job_id,
                root = %job.root_hex,
                bankai_finalized_height = finalized_height,
                required_source_block = job.source_block_number,
                decision = "wait",
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
        db::mark_observed_root_finalized(&self.pool, job.observed_root_id, finalized_height)
            .await?;
        db::update_job_state(&self.pool, job.job_id, ReplicationJobState::ReadyToProve).await?;
        Ok(())
    }

    async fn advance_ready_to_prove(&self, job: ActiveJob) -> Result<()> {
        info!(
            job_id = job.job_id,
            root = %job.root_hex,
            source_block_number = job.source_block_number,
            "fetching exact Bankai proof bundle"
        );
        let bundle_bytes = match self
            .bundle_client
            .fetch_exact_block_bundle(job.source_block_number)
            .await
        {
            Ok(bundle) => {
                info!(
                    job_id = job.job_id,
                    root = %job.root_hex,
                    source_block_number = job.source_block_number,
                    bundle_size_bytes = bundle.len(),
                    "fetched exact Bankai proof bundle"
                );
                bundle
            }
            Err(error) => {
                warn!(
                    ?error,
                    job_id = job.job_id,
                    root = %job.root_hex,
                    source_block_number = job.source_block_number,
                    "failed to fetch exact Bankai proof bundle"
                );
                self.mark_job_retryable_or_failed(
                    &job,
                    ReplicationJobState::ReadyToProve,
                    &error.to_string(),
                )
                .await?;
                return Ok(());
            }
        };

        db::mark_job_proof_in_progress(&self.pool, job.job_id).await?;
        info!(
            job_id = job.job_id,
            root = %job.root_hex,
            source_block_number = job.source_block_number,
            "requesting SP1 proof"
        );

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

                warn!(
                    ?error,
                    job_id = job.job_id,
                    root = %job.root_hex,
                    source_block_number = job.source_block_number,
                    next_state = %next_state,
                    "SP1 proof generation failed"
                );

                if next_state == ReplicationJobState::Failed {
                    db::mark_job_failed(&self.pool, job.job_id, &error.to_string()).await?;
                } else {
                    self.mark_job_retryable_or_failed(
                        &job,
                        ReplicationJobState::ReadyToProve,
                        &error.to_string(),
                    )
                    .await?;
                }

                return Ok(());
            }
        };

        db::mark_job_proof_ready(&self.pool, job.job_id, &proof_artifact.path).await?;
        info!(
            job_id = job.job_id,
            root = %job.root_hex,
            source_block_number = job.source_block_number,
            artifact_path = %proof_artifact.path,
            "SP1 proof artifact is ready"
        );
        Ok(())
    }

    async fn advance_submissions(&self, job: ActiveJob) -> Result<()> {
        let submissions = db::job_submissions(&self.pool, job.job_id).await?;
        if submissions.is_empty() {
            db::mark_job_failed(&self.pool, job.job_id, "no chain submissions were created")
                .await?;
            return Ok(());
        }

        let Some(artifact_path) = job.proof_artifact_ref.as_deref() else {
            self.mark_job_retryable_or_failed(
                &job,
                ReplicationJobState::ReadyToProve,
                "missing proof artifact ref for submission",
            )
            .await?;
            return Ok(());
        };

        if submissions
            .iter()
            .any(|submission| submission.submission_state == ChainSubmissionState::Pending)
        {
            match self.proof_service.load(artifact_path).await {
                Ok(proof_artifact) => {
                    if proof_artifact.decoded_public_values.source_block_number
                        != job.source_block_number
                        || root_to_hex(proof_artifact.decoded_public_values.root) != job.root_hex
                    {
                        let message = "proof artifact public values do not match the observed root";
                        self.mark_unfinished_submissions_failed(&submissions, message)
                            .await?;
                        db::mark_job_failed(&self.pool, job.job_id, message).await?;
                        return Ok(());
                    }
                }
                Err(error) => {
                    db::mark_job_failed(&self.pool, job.job_id, &error.to_string()).await?;
                    self.mark_unfinished_submissions_failed(&submissions, &error.to_string())
                        .await?;
                    return Ok(());
                }
            }
        }

        for submission in submissions {
            match submission.submission_state {
                ChainSubmissionState::Pending => {
                    self.advance_pending_submission(job.job_id, artifact_path, &submission)
                        .await?;
                }
                ChainSubmissionState::Submitting => {
                    self.advance_submitting_submission(job.job_id, &submission)
                        .await?;
                }
                ChainSubmissionState::Confirmed | ChainSubmissionState::Failed => {}
            }
        }

        self.reconcile_submission_job(job.job_id).await
    }

    async fn advance_pending_submission(
        &self,
        job_id: i64,
        artifact_path: &str,
        submission: &ChainSubmission,
    ) -> Result<()> {
        let client = self.submission_client(&submission.chain_name)?;
        match client
            .submit_artifact(&submission.registry_address, artifact_path)
            .await
        {
            Ok(tx_hash) => {
                db::mark_submission_submitting(&self.pool, submission.submission_id, &tx_hash)
                    .await?;
                info!(
                    job_id,
                    chain = %submission.chain_name,
                    %tx_hash,
                    "submitted proof"
                );
            }
            Err(error) => {
                warn!(
                    ?error,
                    job_id,
                    chain = %submission.chain_name,
                    "failed to submit proof"
                );
                self.mark_submission_retryable_or_failed(
                    submission,
                    ChainSubmissionState::Pending,
                    &error.to_string(),
                )
                .await?;
            }
        }

        Ok(())
    }

    async fn advance_submitting_submission(
        &self,
        job_id: i64,
        submission: &ChainSubmission,
    ) -> Result<()> {
        let Some(tx_hash) = submission.submission_tx_hash.as_deref() else {
            self.mark_submission_retryable_or_failed(
                submission,
                ChainSubmissionState::Pending,
                "submission entered submitting state without a transaction hash",
            )
            .await?;
            return Ok(());
        };

        let client = self.submission_client(&submission.chain_name)?;
        match client.check_submission(tx_hash).await {
            Ok(SubmissionCheck::Pending) => Ok(()),
            Ok(SubmissionCheck::Confirmed) => {
                db::mark_submission_confirmed(&self.pool, submission.submission_id, tx_hash)
                    .await?;
                Ok(())
            }
            Ok(SubmissionCheck::Failed(message)) => {
                warn!(
                    job_id,
                    chain = %submission.chain_name,
                    %message,
                    "submission failed"
                );
                db::mark_submission_failed(&self.pool, submission.submission_id, &message).await?;
                Ok(())
            }
            Err(error) => {
                warn!(
                    ?error,
                    job_id,
                    chain = %submission.chain_name,
                    "failed to confirm submission"
                );
                self.mark_submission_retryable_or_failed(
                    submission,
                    ChainSubmissionState::Submitting,
                    &error.to_string(),
                )
                .await?;
                Ok(())
            }
        }
    }

    async fn reconcile_submission_job(&self, job_id: i64) -> Result<()> {
        let submissions = db::job_submissions(&self.pool, job_id).await?;
        if submissions.is_empty() {
            db::mark_job_failed(&self.pool, job_id, "no chain submissions were created").await?;
            return Ok(());
        }

        let any_active = submissions.iter().any(|submission| {
            matches!(
                submission.submission_state,
                ChainSubmissionState::Pending | ChainSubmissionState::Submitting
            )
        });
        if any_active {
            db::update_job_state(&self.pool, job_id, ReplicationJobState::Submitting).await?;
            return Ok(());
        }

        let all_confirmed = submissions
            .iter()
            .all(|submission| submission.submission_state == ChainSubmissionState::Confirmed);
        if all_confirmed {
            db::mark_job_completed(&self.pool, job_id).await?;
            return Ok(());
        }

        let failed_chains = submissions
            .iter()
            .filter(|submission| submission.submission_state == ChainSubmissionState::Failed)
            .map(|submission| submission.chain_name.clone())
            .collect::<Vec<_>>();

        db::mark_job_failed(
            &self.pool,
            job_id,
            &format!(
                "one or more chain submissions failed: {}",
                failed_chains.join(", ")
            ),
        )
        .await?;
        Ok(())
    }

    async fn mark_unfinished_submissions_failed(
        &self,
        submissions: &[ChainSubmission],
        message: &str,
    ) -> Result<()> {
        for submission in submissions {
            if submission.submission_state != ChainSubmissionState::Confirmed {
                db::mark_submission_failed(&self.pool, submission.submission_id, message).await?;
            }
        }

        Ok(())
    }

    fn submission_client(&self, chain_name: &str) -> Result<Arc<dyn SubmissionClient>> {
        self.submission_clients
            .get(chain_name)
            .cloned()
            .with_context(|| format!("missing submission client for {chain_name}"))
    }

    async fn mark_job_retryable_or_failed(
        &self,
        job: &ActiveJob,
        state: ReplicationJobState,
        message: &str,
    ) -> Result<()> {
        if job.job_retry_count + 1 >= MAX_JOB_RPC_FAILURES {
            db::mark_job_failed_after_retry(
                &self.pool,
                job.job_id,
                &retry_limit_message(MAX_JOB_RPC_FAILURES, message),
            )
            .await?;
            return Ok(());
        }

        db::mark_job_retryable(&self.pool, job.job_id, state, message).await
    }

    async fn mark_submission_retryable_or_failed(
        &self,
        submission: &ChainSubmission,
        state: ChainSubmissionState,
        message: &str,
    ) -> Result<()> {
        if submission.submission_retry_count + 1 >= MAX_SUBMISSION_RPC_FAILURES {
            db::mark_submission_failed_after_retry(
                &self.pool,
                submission.submission_id,
                &retry_limit_message(MAX_SUBMISSION_RPC_FAILURES, message),
            )
            .await?;
            return Ok(());
        }

        db::mark_submission_retryable(&self.pool, submission.submission_id, state, message).await
    }
}

fn is_terminal_proving_error(error: &anyhow::Error) -> bool {
    error
        .to_string()
        .contains("decoded proof public values do not match observed root")
}

fn retry_limit_message(limit: u32, message: &str) -> String {
    format!("retry limit reached after {limit} attempts: {message}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::jobs::types::{DestinationChain, ObservedRoot};
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
            _destinations: &[DestinationChainConfig],
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
        submit_error: Option<String>,
        check_results: Mutex<Vec<Result<SubmissionCheck>>>,
    }

    #[async_trait]
    impl SubmissionClient for FakeSubmissionClient {
        async fn submit_artifact(
            &self,
            _contract_address: &str,
            artifact_path: &str,
        ) -> Result<String> {
            if let Some(error) = &self.submit_error {
                anyhow::bail!(error.clone());
            }

            self.submitted
                .lock()
                .unwrap()
                .push(artifact_path.to_string());
            Ok("0x1111111111111111111111111111111111111111111111111111111111111111".to_string())
        }

        async fn check_submission(&self, _tx_hash: &str) -> Result<SubmissionCheck> {
            let mut results = self.check_results.lock().unwrap();
            if results.is_empty() {
                return Ok(SubmissionCheck::Confirmed);
            }

            results.remove(0)
        }
    }

    #[tokio::test]
    async fn runner_waits_for_bankai_finality() {
        let pool = test_pool().await;
        let destinations = vec![destination(DestinationChain::BaseSepolia)];
        record_root(&pool, &destinations, [5u8; 32], 100, "0xabc").await;

        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations.clone(),
            fake_clients_for(&destinations),
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 99 }),
            Arc::new(StaticBundleClient),
            Arc::new(FakeProofService {
                prove_calls: AtomicU64::new(0),
            }),
        );

        runner.advance_once().await.unwrap();

        let job = db::next_active_job(&pool).await.unwrap().unwrap();
        assert_eq!(job.job_state, ReplicationJobState::WaitingFinality);
    }

    #[tokio::test]
    async fn recording_the_same_root_twice_reuses_the_same_job() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        let observed_root = ObservedRoot {
            root_hex: format!("0x{}", hex::encode([9u8; 32])),
            source_block_number: 77,
            source_tx_hash: "0xaaa".to_string(),
        };

        let created = db::record_observed_root(&pool, &observed_root, &destinations, false)
            .await
            .unwrap();
        let created_again = db::record_observed_root(&pool, &observed_root, &destinations, false)
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
        let submission_count =
            sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM chain_submissions")
                .fetch_one(&pool)
                .await
                .unwrap();

        assert!(created.created);
        assert!(!created_again.created);
        assert_eq!(observed_root_count, 1);
        assert_eq!(job_count, 1);
        assert_eq!(submission_count, destinations.len() as i64);
    }

    #[tokio::test]
    async fn recording_a_new_root_replaces_older_pending_roots() {
        let pool = test_pool().await;
        let destinations = all_destinations();

        record_root(&pool, &destinations, [1u8; 32], 10, "0x111").await;

        let created = db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([2u8; 32])),
                source_block_number: 20,
                source_tx_hash: "0x222".to_string(),
            },
            &destinations,
            false,
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
        let destinations = vec![destination(DestinationChain::BaseSepolia)];
        record_root(&pool, &destinations, [3u8; 32], 55, "0xbbb").await;

        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations.clone(),
            fake_clients_for(&destinations),
            Arc::new(NoopWatcher),
            Arc::new(FailingFinalityClient),
            Arc::new(StaticBundleClient),
            Arc::new(FakeProofService {
                prove_calls: AtomicU64::new(0),
            }),
        );

        runner.advance_once().await.unwrap();

        let job = db::next_active_job(&pool).await.unwrap().unwrap();
        assert_eq!(job.job_state, ReplicationJobState::WaitingFinality);
        assert_eq!(job.job_retry_count, 1);
        assert_eq!(
            job.job_error_message.as_deref(),
            Some("bankai finality unavailable")
        );
    }

    #[tokio::test]
    async fn runner_advances_newer_jobs_even_when_an_older_submission_is_stuck() {
        let pool = test_pool().await;
        let destinations = vec![destination(DestinationChain::BaseSepolia)];

        record_root(&pool, &destinations, [3u8; 32], 10, "0xold").await;
        let old_job = db::next_active_job(&pool).await.unwrap().unwrap();
        db::mark_job_proof_ready(&pool, old_job.job_id, "/tmp/fake-proof-old")
            .await
            .unwrap();
        let old_submission = db::job_submissions(&pool, old_job.job_id)
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();
        db::mark_submission_submitting(
            &pool,
            old_submission.submission_id,
            "0x1111111111111111111111111111111111111111111111111111111111111111",
        )
        .await
        .unwrap();
        db::update_job_state(&pool, old_job.job_id, ReplicationJobState::Submitting)
            .await
            .unwrap();

        record_root(&pool, &destinations, [4u8; 32], 20, "0xnew").await;
        let new_job_id = sqlx::query_scalar::<_, i64>(
            "SELECT id FROM replication_jobs WHERE observed_root_id != ? ORDER BY id DESC LIMIT 1",
        )
        .bind(old_job.observed_root_id)
        .fetch_one(&pool)
        .await
        .unwrap();

        let base = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations.clone(),
            HashMap::from([("base-sepolia", base as Arc<dyn SubmissionClient>)]),
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 20 }),
            Arc::new(StaticBundleClient),
            Arc::new(FakeProofService {
                prove_calls: AtomicU64::new(0),
            }),
        );

        runner.advance_once().await.unwrap();

        let new_job = db::job_snapshot(&pool, new_job_id).await.unwrap().unwrap();
        assert_eq!(new_job.job_state, ReplicationJobState::ReadyToProve);
        assert_eq!(new_job.bankai_finalized_block_number, Some(20));
    }

    #[tokio::test]
    async fn runner_proves_once_and_fans_out_to_all_chains() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        record_root(&pool, &destinations, [7u8; 32], 12_345, "0xdef").await;

        let base = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let op = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let arb = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let starknet = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let solana = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let chiado = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let monad = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let hyperevm = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let tempo = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let megaeth = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let plasma = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let submission_clients = HashMap::from([
            ("base-sepolia", base.clone() as Arc<dyn SubmissionClient>),
            ("op-sepolia", op.clone() as Arc<dyn SubmissionClient>),
            ("arbitrum-sepolia", arb.clone() as Arc<dyn SubmissionClient>),
            (
                "starknet-sepolia",
                starknet.clone() as Arc<dyn SubmissionClient>,
            ),
            ("solana-devnet", solana.clone() as Arc<dyn SubmissionClient>),
            ("chiado", chiado.clone() as Arc<dyn SubmissionClient>),
            ("monad-testnet", monad.clone() as Arc<dyn SubmissionClient>),
            (
                "hyperevm-testnet",
                hyperevm.clone() as Arc<dyn SubmissionClient>,
            ),
            ("tempo-testnet", tempo.clone() as Arc<dyn SubmissionClient>),
            (
                "megaeth-testnet",
                megaeth.clone() as Arc<dyn SubmissionClient>,
            ),
            (
                "plasma-testnet",
                plasma.clone() as Arc<dyn SubmissionClient>,
            ),
        ]);
        let proof_service = Arc::new(FakeProofService {
            prove_calls: AtomicU64::new(0),
        });

        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations,
            submission_clients,
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            proof_service.clone(),
        );

        runner.advance_once().await.unwrap();
        assert_eq!(
            db::next_active_job(&pool).await.unwrap().unwrap().job_state,
            ReplicationJobState::ReadyToProve
        );

        runner.advance_once().await.unwrap();
        assert_eq!(
            db::next_active_job(&pool).await.unwrap().unwrap().job_state,
            ReplicationJobState::ProofReady
        );

        runner.advance_once().await.unwrap();
        assert_eq!(
            db::next_active_job(&pool).await.unwrap().unwrap().job_state,
            ReplicationJobState::Submitting
        );
        assert_eq!(base.submitted.lock().unwrap().len(), 1);
        assert_eq!(op.submitted.lock().unwrap().len(), 1);
        assert_eq!(arb.submitted.lock().unwrap().len(), 1);
        assert_eq!(starknet.submitted.lock().unwrap().len(), 1);
        assert_eq!(solana.submitted.lock().unwrap().len(), 1);
        assert_eq!(chiado.submitted.lock().unwrap().len(), 1);
        assert_eq!(monad.submitted.lock().unwrap().len(), 1);
        assert_eq!(hyperevm.submitted.lock().unwrap().len(), 1);
        assert_eq!(tempo.submitted.lock().unwrap().len(), 1);
        assert_eq!(megaeth.submitted.lock().unwrap().len(), 1);
        assert_eq!(plasma.submitted.lock().unwrap().len(), 1);

        runner.advance_once().await.unwrap();
        assert!(db::next_active_job(&pool).await.unwrap().is_none());
        assert_eq!(proof_service.prove_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn chain_failure_does_not_block_other_chain_success() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        record_root(&pool, &destinations, [7u8; 32], 12_345, "0xdef").await;

        let base = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let op = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: Some("op transport error".to_string()),
            check_results: Mutex::new(vec![]),
        });
        let arb = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let starknet = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let solana = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let chiado = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let monad = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let hyperevm = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let tempo = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let megaeth = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let plasma = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let submission_clients = HashMap::from([
            ("base-sepolia", base.clone() as Arc<dyn SubmissionClient>),
            ("op-sepolia", op.clone() as Arc<dyn SubmissionClient>),
            ("arbitrum-sepolia", arb.clone() as Arc<dyn SubmissionClient>),
            (
                "starknet-sepolia",
                starknet.clone() as Arc<dyn SubmissionClient>,
            ),
            ("solana-devnet", solana.clone() as Arc<dyn SubmissionClient>),
            ("chiado", chiado.clone() as Arc<dyn SubmissionClient>),
            ("monad-testnet", monad.clone() as Arc<dyn SubmissionClient>),
            (
                "hyperevm-testnet",
                hyperevm.clone() as Arc<dyn SubmissionClient>,
            ),
            ("tempo-testnet", tempo.clone() as Arc<dyn SubmissionClient>),
            (
                "megaeth-testnet",
                megaeth.clone() as Arc<dyn SubmissionClient>,
            ),
            (
                "plasma-testnet",
                plasma.clone() as Arc<dyn SubmissionClient>,
            ),
        ]);

        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations,
            submission_clients,
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            Arc::new(FakeProofService {
                prove_calls: AtomicU64::new(0),
            }),
        );

        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();

        let submissions = db::job_submissions(&pool, 1).await.unwrap();
        assert_eq!(
            submission_state(&submissions, "base-sepolia"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "op-sepolia"),
            ChainSubmissionState::Pending
        );
        assert_eq!(
            submission_state(&submissions, "arbitrum-sepolia"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "starknet-sepolia"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "solana-devnet"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "chiado"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "monad-testnet"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "hyperevm-testnet"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "tempo-testnet"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "megaeth-testnet"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(
            submission_state(&submissions, "plasma-testnet"),
            ChainSubmissionState::Submitting
        );
        assert_eq!(base.submitted.lock().unwrap().len(), 1);
        assert_eq!(arb.submitted.lock().unwrap().len(), 1);
        assert_eq!(starknet.submitted.lock().unwrap().len(), 1);
        assert_eq!(solana.submitted.lock().unwrap().len(), 1);
        assert_eq!(chiado.submitted.lock().unwrap().len(), 1);
        assert_eq!(monad.submitted.lock().unwrap().len(), 1);
        assert_eq!(hyperevm.submitted.lock().unwrap().len(), 1);
        assert_eq!(tempo.submitted.lock().unwrap().len(), 1);
        assert_eq!(megaeth.submitted.lock().unwrap().len(), 1);
        assert_eq!(plasma.submitted.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn mixed_chain_outcomes_settle_without_resubmitting_successes() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        record_root(&pool, &destinations, [7u8; 32], 12_345, "0xdef").await;

        let base = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let op = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Failed("op revert".to_string()))]),
        });
        let arb = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let starknet = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let solana = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let chiado = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let monad = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let hyperevm = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let tempo = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let megaeth = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let plasma = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let submission_clients = HashMap::from([
            ("base-sepolia", base.clone() as Arc<dyn SubmissionClient>),
            ("op-sepolia", op.clone() as Arc<dyn SubmissionClient>),
            ("arbitrum-sepolia", arb.clone() as Arc<dyn SubmissionClient>),
            (
                "starknet-sepolia",
                starknet.clone() as Arc<dyn SubmissionClient>,
            ),
            ("solana-devnet", solana.clone() as Arc<dyn SubmissionClient>),
            ("chiado", chiado.clone() as Arc<dyn SubmissionClient>),
            ("monad-testnet", monad.clone() as Arc<dyn SubmissionClient>),
            (
                "hyperevm-testnet",
                hyperevm.clone() as Arc<dyn SubmissionClient>,
            ),
            ("tempo-testnet", tempo.clone() as Arc<dyn SubmissionClient>),
            (
                "megaeth-testnet",
                megaeth.clone() as Arc<dyn SubmissionClient>,
            ),
            (
                "plasma-testnet",
                plasma.clone() as Arc<dyn SubmissionClient>,
            ),
        ]);

        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations,
            submission_clients,
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            Arc::new(FakeProofService {
                prove_calls: AtomicU64::new(0),
            }),
        );

        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();

        assert!(db::next_active_job(&pool).await.unwrap().is_none());

        let job_state =
            sqlx::query_scalar::<_, String>("SELECT state FROM replication_jobs WHERE id = 1")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(job_state, ReplicationJobState::Failed.as_db_str());

        let submissions = db::job_submissions(&pool, 1).await.unwrap();
        assert_eq!(
            submission_state(&submissions, "base-sepolia"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "op-sepolia"),
            ChainSubmissionState::Failed
        );
        assert_eq!(
            submission_state(&submissions, "arbitrum-sepolia"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "starknet-sepolia"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "solana-devnet"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "chiado"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "monad-testnet"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "hyperevm-testnet"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "tempo-testnet"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "megaeth-testnet"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(
            submission_state(&submissions, "plasma-testnet"),
            ChainSubmissionState::Confirmed
        );
        assert_eq!(base.submitted.lock().unwrap().len(), 1);
        assert_eq!(op.submitted.lock().unwrap().len(), 1);
        assert_eq!(arb.submitted.lock().unwrap().len(), 1);
        assert_eq!(starknet.submitted.lock().unwrap().len(), 1);
        assert_eq!(solana.submitted.lock().unwrap().len(), 1);
        assert_eq!(chiado.submitted.lock().unwrap().len(), 1);
        assert_eq!(monad.submitted.lock().unwrap().len(), 1);
        assert_eq!(hyperevm.submitted.lock().unwrap().len(), 1);
        assert_eq!(tempo.submitted.lock().unwrap().len(), 1);
        assert_eq!(megaeth.submitted.lock().unwrap().len(), 1);
        assert_eq!(plasma.submitted.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn runner_does_not_resubmit_confirmed_chains_after_restart() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        record_root(&pool, &destinations, [7u8; 32], 12_345, "0xdef").await;

        let base = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![]),
        });
        let op = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let arb = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let starknet = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let solana = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let chiado = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let monad = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let hyperevm = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let tempo = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let megaeth = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let plasma = Arc::new(FakeSubmissionClient {
            submitted: Mutex::new(Vec::new()),
            submit_error: None,
            check_results: Mutex::new(vec![Ok(SubmissionCheck::Pending)]),
        });
        let submission_clients = HashMap::from([
            ("base-sepolia", base.clone() as Arc<dyn SubmissionClient>),
            ("op-sepolia", op.clone() as Arc<dyn SubmissionClient>),
            ("arbitrum-sepolia", arb.clone() as Arc<dyn SubmissionClient>),
            (
                "starknet-sepolia",
                starknet.clone() as Arc<dyn SubmissionClient>,
            ),
            ("solana-devnet", solana.clone() as Arc<dyn SubmissionClient>),
            ("chiado", chiado.clone() as Arc<dyn SubmissionClient>),
            ("monad-testnet", monad.clone() as Arc<dyn SubmissionClient>),
            (
                "hyperevm-testnet",
                hyperevm.clone() as Arc<dyn SubmissionClient>,
            ),
            ("tempo-testnet", tempo.clone() as Arc<dyn SubmissionClient>),
            (
                "megaeth-testnet",
                megaeth.clone() as Arc<dyn SubmissionClient>,
            ),
            (
                "plasma-testnet",
                plasma.clone() as Arc<dyn SubmissionClient>,
            ),
        ]);
        let proof_service = Arc::new(FakeProofService {
            prove_calls: AtomicU64::new(0),
        });

        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations.clone(),
            submission_clients.clone(),
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            proof_service.clone(),
        );

        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();

        assert_eq!(base.submitted.lock().unwrap().len(), 1);
        assert_eq!(op.submitted.lock().unwrap().len(), 1);
        assert_eq!(arb.submitted.lock().unwrap().len(), 1);
        assert_eq!(starknet.submitted.lock().unwrap().len(), 1);
        assert_eq!(solana.submitted.lock().unwrap().len(), 1);
        assert_eq!(chiado.submitted.lock().unwrap().len(), 1);
        assert_eq!(monad.submitted.lock().unwrap().len(), 1);
        assert_eq!(hyperevm.submitted.lock().unwrap().len(), 1);
        assert_eq!(tempo.submitted.lock().unwrap().len(), 1);
        assert_eq!(megaeth.submitted.lock().unwrap().len(), 1);
        assert_eq!(plasma.submitted.lock().unwrap().len(), 1);

        let restarted = Runner::new_for_tests(
            pool.clone(),
            destinations,
            submission_clients,
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            proof_service.clone(),
        );

        restarted.advance_once().await.unwrap();

        assert_eq!(base.submitted.lock().unwrap().len(), 1);
        assert_eq!(op.submitted.lock().unwrap().len(), 1);
        assert_eq!(arb.submitted.lock().unwrap().len(), 1);
        assert_eq!(starknet.submitted.lock().unwrap().len(), 1);
        assert_eq!(solana.submitted.lock().unwrap().len(), 1);
        assert_eq!(chiado.submitted.lock().unwrap().len(), 1);
        assert_eq!(monad.submitted.lock().unwrap().len(), 1);
        assert_eq!(hyperevm.submitted.lock().unwrap().len(), 1);
        assert_eq!(tempo.submitted.lock().unwrap().len(), 1);
        assert_eq!(megaeth.submitted.lock().unwrap().len(), 1);
        assert_eq!(plasma.submitted.lock().unwrap().len(), 1);
        assert_eq!(proof_service.prove_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn recent_proof_requests_do_not_block_new_jobs_during_dev() {
        let pool = test_pool().await;
        let destinations = vec![destination(DestinationChain::BaseSepolia)];
        let proof_service = Arc::new(FakeProofService {
            prove_calls: AtomicU64::new(0),
        });

        record_root(&pool, &destinations, [3u8; 32], 100, "0xold").await;
        sqlx::query(
            r#"
            UPDATE replication_jobs
            SET
                state = ?,
                proof_requested_at = datetime('now', '-51 minutes')
            "#,
        )
        .bind(ReplicationJobState::Completed.as_db_str())
        .execute(&pool)
        .await
        .unwrap();

        let created = db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([7u8; 32])),
                source_block_number: 12_345,
                source_tx_hash: "0xnew".to_string(),
            },
            &destinations,
            false,
        )
        .await
        .unwrap();
        assert!(created.created);
        assert!(!created.skipped);

        let runner = Runner::new_for_tests(
            pool.clone(),
            destinations.clone(),
            fake_clients_for(&destinations),
            Arc::new(NoopWatcher),
            Arc::new(StaticFinalityClient { height: 12_345 }),
            Arc::new(StaticBundleClient),
            proof_service.clone(),
        );

        runner.advance_once().await.unwrap();
        runner.advance_once().await.unwrap();

        let job = db::next_active_job(&pool).await.unwrap().unwrap();
        assert_eq!(job.job_state, ReplicationJobState::ProofReady);
        assert_eq!(proof_service.prove_calls.load(Ordering::Relaxed), 1);
    }

    #[tokio::test]
    async fn recent_proof_requests_skip_new_jobs_when_gap_is_enabled() {
        let pool = test_pool().await;
        let destinations = vec![destination(DestinationChain::BaseSepolia)];

        record_root(&pool, &destinations, [3u8; 32], 100, "0xold").await;
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

        let created = db::record_observed_root(
            &pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode([7u8; 32])),
                source_block_number: 12_345,
                source_tx_hash: "0xnew".to_string(),
            },
            &destinations,
            true,
        )
        .await
        .unwrap();

        let skipped_status = sqlx::query_scalar::<_, String>(
            r#"
            SELECT status
            FROM observed_roots
            WHERE source_block_number = ?
            "#,
        )
        .bind(12_345_i64)
        .fetch_one(&pool)
        .await
        .unwrap();

        assert!(created.created);
        assert!(created.skipped);
        assert_eq!(created.replaced_pending_count, 0);
        assert_eq!(skipped_status, "skipped");
        assert!(db::next_active_job(&pool).await.unwrap().is_none());
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

    async fn record_root(
        pool: &SqlitePool,
        destinations: &[DestinationChainConfig],
        root: [u8; 32],
        source_block_number: u64,
        tx_hash: &str,
    ) {
        db::record_observed_root(
            pool,
            &ObservedRoot {
                root_hex: format!("0x{}", hex::encode(root)),
                source_block_number,
                source_tx_hash: tx_hash.to_string(),
            },
            destinations,
            false,
        )
        .await
        .unwrap();
    }

    fn all_destinations() -> Vec<DestinationChainConfig> {
        vec![
            destination(DestinationChain::BaseSepolia),
            destination(DestinationChain::OpSepolia),
            destination(DestinationChain::ArbitrumSepolia),
            destination(DestinationChain::StarknetSepolia),
            destination(DestinationChain::SolanaDevnet),
            destination(DestinationChain::Chiado),
            destination(DestinationChain::MonadTestnet),
            destination(DestinationChain::HyperEvmTestnet),
            destination(DestinationChain::TempoTestnet),
            destination(DestinationChain::MegaEthTestnet),
            destination(DestinationChain::PlasmaTestnet),
        ]
    }

    fn destination(chain: DestinationChain) -> DestinationChainConfig {
        let suffix = match chain {
            DestinationChain::BaseSepolia => "0123",
            DestinationChain::OpSepolia => "0456",
            DestinationChain::ArbitrumSepolia => "0789",
            DestinationChain::StarknetSepolia => "0abc",
            DestinationChain::SolanaDevnet => "solana",
            DestinationChain::Chiado => "1020",
            DestinationChain::MonadTestnet => "1143",
            DestinationChain::HyperEvmTestnet => "0998",
            DestinationChain::TempoTestnet => "2431",
            DestinationChain::MegaEthTestnet => "6343",
            DestinationChain::PlasmaTestnet => "9746",
        };

        DestinationChainConfig {
            chain,
            rpc_url: "https://example.invalid".to_string(),
            contract_address: match chain {
                DestinationChain::StarknetSepolia => {
                    "0x04f213f87dd6eec0951c49ec9e2d577fabf843d7e022f33d04e6a25ff8954e61".to_string()
                }
                DestinationChain::SolanaDevnet => {
                    "HpgNxwdekXixEW6ZzTPsjhhFx46fpfoC7ruJvsinPYHx".to_string()
                }
                _ => format!("0x000000000000000000000000000000000000{suffix}"),
            },
            private_key: "0x01".to_string(),
            account_address: if chain == DestinationChain::StarknetSepolia {
                Some(
                    "0x04f213f87dd6eec0951c49ec9e2d577fabf843d7e022f33d04e6a25ff8954e61"
                        .to_string(),
                )
            } else {
                None
            },
        }
    }

    fn fake_clients_for(
        destinations: &[DestinationChainConfig],
    ) -> HashMap<&'static str, Arc<dyn SubmissionClient>> {
        destinations
            .iter()
            .map(|destination| {
                (
                    destination.name(),
                    Arc::new(FakeSubmissionClient {
                        submitted: Mutex::new(Vec::new()),
                        submit_error: None,
                        check_results: Mutex::new(vec![]),
                    }) as Arc<dyn SubmissionClient>,
                )
            })
            .collect()
    }

    fn submission_state(submissions: &[ChainSubmission], chain_name: &str) -> ChainSubmissionState {
        submissions
            .iter()
            .find(|submission| submission.chain_name == chain_name)
            .unwrap()
            .submission_state
    }
}
