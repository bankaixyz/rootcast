use crate::config::DestinationChainConfig;
use crate::jobs::types::{ChainSubmissionState, ObservedRoot, ReplicationJobState};
use anyhow::{Context, Result};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{FromRow, SqlitePool};
use std::str::FromStr;

const MIN_PROOF_REQUEST_AGE_SECS: i64 = 50 * 60;

pub async fn connect(database_url: &str) -> Result<SqlitePool> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    Ok(pool)
}

pub async fn migrate(pool: &SqlitePool) -> Result<()> {
    sqlx::migrate!("./migrations").run(pool).await?;
    Ok(())
}

#[derive(Clone, Debug, FromRow)]
pub struct ActiveJobRow {
    pub job_id: i64,
    pub observed_root_id: i64,
    pub root_hex: String,
    pub source_block_number: i64,
    pub source_tx_hash: String,
    pub job_state: String,
    pub proof_requested_at: Option<String>,
    pub proof_artifact_ref: Option<String>,
    pub job_error_message: Option<String>,
    pub job_retry_count: i64,
}

#[derive(Clone, Debug)]
pub struct ActiveJob {
    pub job_id: i64,
    pub observed_root_id: i64,
    pub root_hex: String,
    pub source_block_number: u64,
    pub source_tx_hash: String,
    pub job_state: ReplicationJobState,
    pub proof_requested_at: Option<String>,
    pub proof_artifact_ref: Option<String>,
    pub job_error_message: Option<String>,
    pub job_retry_count: u32,
}

#[derive(Clone, Debug, FromRow)]
pub struct JobSnapshotRow {
    pub job_id: i64,
    pub observed_root_id: i64,
    pub root_hex: String,
    pub source_block_number: i64,
    pub source_tx_hash: String,
    pub observed_at: String,
    pub bankai_finalized_at: Option<String>,
    pub bankai_finalized_block_number: Option<i64>,
    pub observed_root_status: String,
    pub job_state: String,
    pub proof_requested_at: Option<String>,
    pub proof_artifact_ref: Option<String>,
    pub job_error_message: Option<String>,
    pub job_retry_count: i64,
}

#[derive(Clone, Debug)]
pub struct JobSnapshot {
    pub job_id: i64,
    pub observed_root_id: i64,
    pub root_hex: String,
    pub source_block_number: u64,
    pub source_tx_hash: String,
    pub observed_at: String,
    pub bankai_finalized_at: Option<String>,
    pub bankai_finalized_block_number: Option<u64>,
    pub observed_root_status: String,
    pub job_state: ReplicationJobState,
    pub proof_requested_at: Option<String>,
    pub proof_artifact_ref: Option<String>,
    pub job_error_message: Option<String>,
    pub job_retry_count: u32,
}

impl TryFrom<ActiveJobRow> for ActiveJob {
    type Error = anyhow::Error;

    fn try_from(row: ActiveJobRow) -> Result<Self> {
        Ok(Self {
            job_id: row.job_id,
            observed_root_id: row.observed_root_id,
            root_hex: row.root_hex,
            source_block_number: u64::try_from(row.source_block_number)
                .context("source_block_number does not fit in u64")?,
            source_tx_hash: row.source_tx_hash,
            job_state: row.job_state.parse()?,
            proof_requested_at: row.proof_requested_at,
            proof_artifact_ref: row.proof_artifact_ref,
            job_error_message: row.job_error_message,
            job_retry_count: u32::try_from(row.job_retry_count)
                .context("job_retry_count does not fit in u32")?,
        })
    }
}

impl TryFrom<JobSnapshotRow> for JobSnapshot {
    type Error = anyhow::Error;

    fn try_from(row: JobSnapshotRow) -> Result<Self> {
        Ok(Self {
            job_id: row.job_id,
            observed_root_id: row.observed_root_id,
            root_hex: row.root_hex,
            source_block_number: u64::try_from(row.source_block_number)
                .context("source_block_number does not fit in u64")?,
            source_tx_hash: row.source_tx_hash,
            observed_at: row.observed_at,
            bankai_finalized_at: row.bankai_finalized_at,
            bankai_finalized_block_number: row
                .bankai_finalized_block_number
                .map(|value| {
                    u64::try_from(value)
                        .context("bankai_finalized_block_number does not fit in u64")
                })
                .transpose()?,
            observed_root_status: row.observed_root_status,
            job_state: row.job_state.parse()?,
            proof_requested_at: row.proof_requested_at,
            proof_artifact_ref: row.proof_artifact_ref,
            job_error_message: row.job_error_message,
            job_retry_count: u32::try_from(row.job_retry_count)
                .context("job_retry_count does not fit in u32")?,
        })
    }
}

#[derive(Clone, Debug, FromRow)]
pub struct ChainSubmissionRow {
    pub submission_id: i64,
    pub chain_name: String,
    pub chain_id: i64,
    pub target_address: String,
    pub submission_state: String,
    pub submission_tx_hash: Option<String>,
    pub submission_error_message: Option<String>,
    pub submission_retry_count: i64,
}

#[derive(Clone, Debug)]
pub struct ChainSubmission {
    pub submission_id: i64,
    pub chain_name: String,
    pub chain_id: u64,
    pub target_address: String,
    pub submission_state: ChainSubmissionState,
    pub submission_tx_hash: Option<String>,
    pub submission_error_message: Option<String>,
    pub submission_retry_count: u32,
}

impl TryFrom<ChainSubmissionRow> for ChainSubmission {
    type Error = anyhow::Error;

    fn try_from(row: ChainSubmissionRow) -> Result<Self> {
        Ok(Self {
            submission_id: row.submission_id,
            chain_name: row.chain_name,
            chain_id: u64::try_from(row.chain_id).context("chain_id does not fit in u64")?,
            target_address: row.target_address,
            submission_state: row.submission_state.parse()?,
            submission_tx_hash: row.submission_tx_hash,
            submission_error_message: row.submission_error_message,
            submission_retry_count: u32::try_from(row.submission_retry_count)
                .context("submission_retry_count does not fit in u32")?,
        })
    }
}

#[derive(Clone, Debug)]
pub struct RecordObservedRootResult {
    pub created: bool,
    pub skipped: bool,
    pub replaced_pending_count: u64,
}

#[derive(Clone, Debug, FromRow)]
pub struct LastProofRequest {
    pub proof_requested_at: String,
    pub age_seconds: i64,
}

pub async fn latest_observed_source_block(pool: &SqlitePool) -> Result<Option<u64>> {
    let latest =
        sqlx::query_scalar::<_, Option<i64>>("SELECT MAX(source_block_number) FROM observed_roots")
            .fetch_one(pool)
            .await?;

    latest
        .map(|value| u64::try_from(value).context("latest observed source block overflow"))
        .transpose()
}

pub async fn latest_job_snapshot(pool: &SqlitePool) -> Result<Option<JobSnapshot>> {
    let row = sqlx::query_as::<_, JobSnapshotRow>(
        r#"
        SELECT
            j.id AS job_id,
            o.id AS observed_root_id,
            o.root_hex,
            o.source_block_number,
            o.source_tx_hash,
            o.observed_at,
            o.bankai_finalized_at,
            o.bankai_finalized_block_number,
            o.status AS observed_root_status,
            j.state AS job_state,
            j.proof_requested_at,
            j.proof_artifact_ref,
            j.error_message AS job_error_message,
            j.retry_count AS job_retry_count
        FROM replication_jobs j
        INNER JOIN observed_roots o ON o.id = j.observed_root_id
        ORDER BY
            CASE
                WHEN j.state IN (?, ?, ?, ?, ?) THEN 0
                ELSE 1
            END,
            o.source_block_number DESC,
            j.id DESC
        LIMIT 1
        "#,
    )
    .bind(ReplicationJobState::WaitingFinality.as_db_str())
    .bind(ReplicationJobState::ReadyToProve.as_db_str())
    .bind(ReplicationJobState::ProofInProgress.as_db_str())
    .bind(ReplicationJobState::ProofReady.as_db_str())
    .bind(ReplicationJobState::Submitting.as_db_str())
    .fetch_optional(pool)
    .await?;

    row.map(JobSnapshot::try_from).transpose()
}

pub async fn recent_job_snapshots(pool: &SqlitePool, limit: u32) -> Result<Vec<JobSnapshot>> {
    let rows = sqlx::query_as::<_, JobSnapshotRow>(
        r#"
        SELECT
            j.id AS job_id,
            o.id AS observed_root_id,
            o.root_hex,
            o.source_block_number,
            o.source_tx_hash,
            o.observed_at,
            o.bankai_finalized_at,
            o.bankai_finalized_block_number,
            o.status AS observed_root_status,
            j.state AS job_state,
            j.proof_requested_at,
            j.proof_artifact_ref,
            j.error_message AS job_error_message,
            j.retry_count AS job_retry_count
        FROM replication_jobs j
        INNER JOIN observed_roots o ON o.id = j.observed_root_id
        ORDER BY o.source_block_number DESC, j.id DESC
        LIMIT ?
        "#,
    )
    .bind(i64::from(limit))
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(JobSnapshot::try_from)
        .collect::<Result<Vec<_>>>()
}

pub async fn job_snapshot(pool: &SqlitePool, job_id: i64) -> Result<Option<JobSnapshot>> {
    let row = sqlx::query_as::<_, JobSnapshotRow>(
        r#"
        SELECT
            j.id AS job_id,
            o.id AS observed_root_id,
            o.root_hex,
            o.source_block_number,
            o.source_tx_hash,
            o.observed_at,
            o.bankai_finalized_at,
            o.bankai_finalized_block_number,
            o.status AS observed_root_status,
            j.state AS job_state,
            j.proof_requested_at,
            j.proof_artifact_ref,
            j.error_message AS job_error_message,
            j.retry_count AS job_retry_count
        FROM replication_jobs j
        INNER JOIN observed_roots o ON o.id = j.observed_root_id
        WHERE j.id = ?
        LIMIT 1
        "#,
    )
    .bind(job_id)
    .fetch_optional(pool)
    .await?;

    row.map(JobSnapshot::try_from).transpose()
}

pub async fn record_observed_root(
    pool: &SqlitePool,
    root: &ObservedRoot,
    destinations: &[DestinationChainConfig],
    enforce_min_proof_request_gap: bool,
) -> Result<RecordObservedRootResult> {
    if destinations.is_empty() {
        anyhow::bail!("at least one destination chain is required");
    }

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM observed_roots
        WHERE root_hex = ? AND source_block_number = ?
        "#,
    )
    .bind(&root.root_hex)
    .bind(to_i64(root.source_block_number)?)
    .fetch_optional(&mut *tx)
    .await?;

    if existing.is_some() {
        tx.commit().await?;
        return Ok(RecordObservedRootResult {
            created: false,
            skipped: false,
            replaced_pending_count: 0,
        });
    }

    if enforce_min_proof_request_gap {
        let latest_proof_request_age_seconds = sqlx::query_scalar::<_, Option<i64>>(
            r#"
            SELECT CAST((julianday('now') - julianday(proof_requested_at)) * 86400 AS INTEGER)
            FROM replication_jobs
            WHERE proof_requested_at IS NOT NULL
            ORDER BY proof_requested_at DESC
            LIMIT 1
            "#,
        )
        .fetch_optional(&mut *tx)
        .await?
        .flatten();

        if latest_proof_request_age_seconds
            .is_some_and(|age_seconds| age_seconds < MIN_PROOF_REQUEST_AGE_SECS)
        {
            let insert = sqlx::query(
                r#"
                INSERT OR IGNORE INTO observed_roots (
                    root_hex,
                    source_block_number,
                    source_tx_hash,
                    status
                ) VALUES (?, ?, ?, ?)
                "#,
            )
            .bind(&root.root_hex)
            .bind(to_i64(root.source_block_number)?)
            .bind(&root.source_tx_hash)
            .bind("skipped")
            .execute(&mut *tx)
            .await?;

            tx.commit().await?;
            return Ok(RecordObservedRootResult {
                created: insert.rows_affected() > 0,
                skipped: true,
                replaced_pending_count: 0,
            });
        }
    }

    let replaced_pending_count = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)
        FROM replication_jobs
        WHERE state IN (?, ?)
        "#,
    )
    .bind(ReplicationJobState::WaitingFinality.as_db_str())
    .bind(ReplicationJobState::ReadyToProve.as_db_str())
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM observed_roots
        WHERE id IN (
            SELECT observed_root_id
            FROM replication_jobs
            WHERE state IN (?, ?)
        )
        "#,
    )
    .bind(ReplicationJobState::WaitingFinality.as_db_str())
    .bind(ReplicationJobState::ReadyToProve.as_db_str())
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        r#"
        INSERT OR IGNORE INTO observed_roots (
            root_hex,
            source_block_number,
            source_tx_hash,
            status
        ) VALUES (?, ?, ?, ?)
        "#,
    )
    .bind(&root.root_hex)
    .bind(to_i64(root.source_block_number)?)
    .bind(&root.source_tx_hash)
    .bind(ReplicationJobState::WaitingFinality.as_db_str())
    .execute(&mut *tx)
    .await?;

    let observed_root_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM observed_roots
        WHERE root_hex = ? AND source_block_number = ?
        "#,
    )
    .bind(&root.root_hex)
    .bind(to_i64(root.source_block_number)?)
    .fetch_one(&mut *tx)
    .await?;

    let job_insert = sqlx::query(
        r#"
        INSERT OR IGNORE INTO replication_jobs (
            observed_root_id,
            state
        ) VALUES (?, ?)
        "#,
    )
    .bind(observed_root_id)
    .bind(ReplicationJobState::WaitingFinality.as_db_str())
    .execute(&mut *tx)
    .await?;

    let job_id = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT id
        FROM replication_jobs
        WHERE observed_root_id = ?
        ORDER BY id
        LIMIT 1
        "#,
    )
    .bind(observed_root_id)
    .fetch_one(&mut *tx)
    .await?;

    for destination in destinations {
        sqlx::query(
            r#"
            INSERT OR IGNORE INTO chain_submissions (
                replication_job_id,
                chain_name,
                chain_id,
                target_address,
                state
            ) VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(job_id)
        .bind(destination.name())
        .bind(to_i64(destination.chain_id())?)
        .bind(&destination.target_address)
        .bind(ChainSubmissionState::Pending.as_db_str())
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;
    Ok(RecordObservedRootResult {
        created: job_insert.rows_affected() > 0,
        skipped: false,
        replaced_pending_count: u64::try_from(replaced_pending_count)
            .context("replaced_pending_count does not fit in u64")?,
    })
}

pub async fn repair_inflight_jobs(pool: &SqlitePool) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET state = ?, updated_at = CURRENT_TIMESTAMP
        WHERE state = ? AND proof_artifact_ref IS NULL
        "#,
    )
    .bind(ReplicationJobState::ReadyToProve.as_db_str())
    .bind(ReplicationJobState::ProofInProgress.as_db_str())
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET state = ?, updated_at = CURRENT_TIMESTAMP
        WHERE state = ? AND proof_artifact_ref IS NOT NULL
        "#,
    )
    .bind(ReplicationJobState::ProofReady.as_db_str())
    .bind(ReplicationJobState::ProofInProgress.as_db_str())
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn next_active_job(pool: &SqlitePool) -> Result<Option<ActiveJob>> {
    let row = sqlx::query_as::<_, ActiveJobRow>(
        r#"
        SELECT
            j.id AS job_id,
            o.id AS observed_root_id,
            o.root_hex,
            o.source_block_number,
            o.source_tx_hash,
            j.state AS job_state,
            j.proof_requested_at,
            j.proof_artifact_ref,
            j.error_message AS job_error_message,
            j.retry_count AS job_retry_count
        FROM replication_jobs j
        INNER JOIN observed_roots o ON o.id = j.observed_root_id
        WHERE j.state IN (?, ?, ?, ?)
        ORDER BY j.id
        LIMIT 1
        "#,
    )
    .bind(ReplicationJobState::WaitingFinality.as_db_str())
    .bind(ReplicationJobState::ReadyToProve.as_db_str())
    .bind(ReplicationJobState::ProofReady.as_db_str())
    .bind(ReplicationJobState::Submitting.as_db_str())
    .fetch_optional(pool)
    .await?;

    row.map(ActiveJob::try_from).transpose()
}

pub async fn job_submissions(pool: &SqlitePool, job_id: i64) -> Result<Vec<ChainSubmission>> {
    let rows = sqlx::query_as::<_, ChainSubmissionRow>(
        r#"
        SELECT
            id AS submission_id,
            chain_name,
            chain_id,
            target_address,
            state AS submission_state,
            tx_hash AS submission_tx_hash,
            error_message AS submission_error_message,
            retry_count AS submission_retry_count
        FROM chain_submissions
        WHERE replication_job_id = ?
        ORDER BY id
        "#,
    )
    .bind(job_id)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(ChainSubmission::try_from)
        .collect::<Result<Vec<_>>>()
}

pub async fn mark_observed_root_finalized(
    pool: &SqlitePool,
    observed_root_id: i64,
    bankai_finalized_block_number: u64,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE observed_roots
        SET
            bankai_finalized_at = CURRENT_TIMESTAMP,
            bankai_finalized_block_number = ?,
            status = ?
        WHERE id = ?
        "#,
    )
    .bind(
        i64::try_from(bankai_finalized_block_number)
            .context("bankai_finalized_block_number does not fit in i64")?,
    )
    .bind("bankai_finalized")
    .bind(observed_root_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_job_proof_in_progress(pool: &SqlitePool, job_id: i64) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET
            state = ?,
            proof_requested_at = CURRENT_TIMESTAMP,
            error_message = NULL,
            updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(ReplicationJobState::ProofInProgress.as_db_str())
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_job_state(
    pool: &SqlitePool,
    job_id: i64,
    state: ReplicationJobState,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET state = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(state.as_db_str())
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn latest_proof_request(pool: &SqlitePool) -> Result<Option<LastProofRequest>> {
    let row = sqlx::query_as::<_, LastProofRequest>(
        r#"
        SELECT
            proof_requested_at,
            CAST((julianday('now') - julianday(proof_requested_at)) * 86400 AS INTEGER) AS age_seconds
        FROM replication_jobs
        WHERE proof_requested_at IS NOT NULL
        ORDER BY proof_requested_at DESC
        LIMIT 1
        "#,
    )
    .fetch_optional(pool)
    .await?;

    Ok(row)
}

pub async fn mark_job_proof_ready(
    pool: &SqlitePool,
    job_id: i64,
    proof_artifact_ref: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET state = ?, proof_artifact_ref = ?, error_message = NULL, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(ReplicationJobState::ProofReady.as_db_str())
    .bind(proof_artifact_ref)
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_job_completed(pool: &SqlitePool, job_id: i64) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET state = ?, error_message = NULL, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(ReplicationJobState::Completed.as_db_str())
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_job_retryable(
    pool: &SqlitePool,
    job_id: i64,
    state: ReplicationJobState,
    message: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET state = ?, error_message = ?, retry_count = retry_count + 1, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(state.as_db_str())
    .bind(message)
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_job_failed(pool: &SqlitePool, job_id: i64, message: &str) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE replication_jobs
        SET state = ?, error_message = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(ReplicationJobState::Failed.as_db_str())
    .bind(message)
    .bind(job_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_submission_submitting(
    pool: &SqlitePool,
    submission_id: i64,
    tx_hash: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE chain_submissions
        SET state = ?, tx_hash = ?, error_message = NULL, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(ChainSubmissionState::Submitting.as_db_str())
    .bind(tx_hash)
    .bind(submission_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_submission_confirmed(
    pool: &SqlitePool,
    submission_id: i64,
    tx_hash: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE chain_submissions
        SET state = ?, tx_hash = ?, error_message = NULL, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(ChainSubmissionState::Confirmed.as_db_str())
    .bind(tx_hash)
    .bind(submission_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_submission_retryable(
    pool: &SqlitePool,
    submission_id: i64,
    state: ChainSubmissionState,
    message: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE chain_submissions
        SET state = ?, error_message = ?, retry_count = retry_count + 1, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(state.as_db_str())
    .bind(message)
    .bind(submission_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn mark_submission_failed(
    pool: &SqlitePool,
    submission_id: i64,
    message: &str,
) -> Result<()> {
    sqlx::query(
        r#"
        UPDATE chain_submissions
        SET state = ?, error_message = ?, updated_at = CURRENT_TIMESTAMP
        WHERE id = ?
        "#,
    )
    .bind(ChainSubmissionState::Failed.as_db_str())
    .bind(message)
    .bind(submission_id)
    .execute(pool)
    .await?;
    Ok(())
}

fn to_i64(value: u64) -> Result<i64> {
    i64::try_from(value).context("u64 value does not fit in sqlite integer")
}
