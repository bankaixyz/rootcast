use crate::db::{ChainSubmission, JobSnapshot};
use crate::jobs::types::{ChainSubmissionState, ReplicationJobState};
use serde::Serialize;

#[derive(Clone, Debug)]
pub struct ConfiguredChain {
    pub name: &'static str,
    pub chain_id: String,
    pub registry_address: String,
}

#[derive(Serialize)]
pub struct StatusResponse {
    pub phase: &'static str,
    pub service: &'static str,
    pub status: &'static str,
    pub destination_chain_count: usize,
    pub latest_observed_source_block: Option<u64>,
    pub latest_proof_request_age_seconds: Option<i64>,
    pub current_stage_label: Option<&'static str>,
    pub current_source_block_number: Option<u64>,
}

#[derive(Serialize)]
pub struct LatestRootResponse {
    pub snapshot: Option<RootSnapshotResponse>,
}

#[derive(Serialize)]
pub struct RootsResponse {
    pub roots: Vec<RootSnapshotResponse>,
}

#[derive(Serialize)]
pub struct ChainsResponse {
    pub chains: Vec<ChainStatusResponse>,
}

#[derive(Serialize)]
pub struct JobDetailResponse {
    pub job: RootSnapshotResponse,
}

#[derive(Clone, Serialize)]
pub struct RootSnapshotResponse {
    pub job_id: i64,
    pub observed_root_id: i64,
    pub root_hex: String,
    pub source_block_number: u64,
    pub source_tx_hash: String,
    pub observed_at: String,
    pub bankai_finalized_at: Option<String>,
    pub bankai_finalized_block_number: Option<u64>,
    pub observed_root_status: String,
    pub job_state: String,
    pub proof_ready: bool,
    pub replication_triggered: bool,
    pub stage_label: &'static str,
    pub stage_description: String,
    pub blocked_by: Option<&'static str>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub confirmed_target_count: usize,
    pub failed_target_count: usize,
    pub targets: Vec<ReplicationTargetResponse>,
}

#[derive(Clone, Serialize)]
pub struct ReplicationTargetResponse {
    pub chain_name: String,
    pub chain_id: String,
    pub registry_address: String,
    pub submission_state: String,
    pub tx_hash: Option<String>,
    pub error_message: Option<String>,
    pub retry_count: u32,
    pub display_state: &'static str,
    pub blocked_reason: Option<String>,
}

#[derive(Serialize)]
pub struct ChainStatusResponse {
    pub chain_name: &'static str,
    pub chain_id: String,
    pub registry_address: String,
    pub latest_job_id: Option<i64>,
    pub latest_root_hex: Option<String>,
    pub latest_source_block_number: Option<u64>,
    pub submission_state: Option<String>,
    pub display_state: &'static str,
    pub blocked_reason: Option<String>,
    pub tx_hash: Option<String>,
    pub error_message: Option<String>,
}

pub fn status_response(
    destination_chain_count: usize,
    latest_observed_source_block: Option<u64>,
    latest_proof_request_age_seconds: Option<i64>,
    current_snapshot: Option<&JobSnapshot>,
) -> StatusResponse {
    StatusResponse {
        phase: "phase-4-read-only-api-frontend",
        service: "world-id-root-replicator-backend",
        status: "ok",
        destination_chain_count,
        latest_observed_source_block,
        latest_proof_request_age_seconds,
        current_stage_label: current_snapshot.map(|snapshot| stage(snapshot).label),
        current_source_block_number: current_snapshot.map(|snapshot| snapshot.source_block_number),
    }
}

pub fn root_snapshot(
    snapshot: JobSnapshot,
    submissions: Vec<ChainSubmission>,
) -> RootSnapshotResponse {
    let stage = stage(&snapshot);
    let targets = submissions
        .into_iter()
        .map(|submission| replication_target(&snapshot, submission))
        .collect::<Vec<_>>();
    let replication_triggered = targets.iter().any(|target| {
        matches!(
            target.display_state,
            "queued" | "submitting" | "confirmed" | "failed"
        )
    });

    let confirmed_target_count = targets
        .iter()
        .filter(|target| target.display_state == "confirmed")
        .count();
    let failed_target_count = targets
        .iter()
        .filter(|target| target.display_state == "failed")
        .count();

    RootSnapshotResponse {
        job_id: snapshot.job_id,
        observed_root_id: snapshot.observed_root_id,
        root_hex: snapshot.root_hex,
        source_block_number: snapshot.source_block_number,
        source_tx_hash: snapshot.source_tx_hash,
        observed_at: snapshot.observed_at,
        bankai_finalized_at: snapshot.bankai_finalized_at,
        bankai_finalized_block_number: snapshot.bankai_finalized_block_number,
        observed_root_status: snapshot.observed_root_status,
        job_state: snapshot.job_state.as_db_str().to_string(),
        proof_ready: snapshot.proof_artifact_ref.is_some(),
        replication_triggered,
        stage_label: stage.label,
        stage_description: stage.description,
        blocked_by: stage.blocked_by,
        error_message: snapshot.job_error_message,
        retry_count: snapshot.job_retry_count,
        confirmed_target_count,
        failed_target_count,
        targets,
    }
}

pub fn chain_status(
    configured_chain: &ConfiguredChain,
    latest_snapshot: Option<&RootSnapshotResponse>,
) -> ChainStatusResponse {
    let target = latest_snapshot.and_then(|snapshot| {
        snapshot
            .targets
            .iter()
            .find(|target| target.chain_name == configured_chain.name)
    });

    ChainStatusResponse {
        chain_name: configured_chain.name,
        chain_id: configured_chain.chain_id.clone(),
        registry_address: configured_chain.registry_address.clone(),
        latest_job_id: latest_snapshot.map(|snapshot| snapshot.job_id),
        latest_root_hex: latest_snapshot.map(|snapshot| snapshot.root_hex.clone()),
        latest_source_block_number: latest_snapshot.map(|snapshot| snapshot.source_block_number),
        submission_state: target.map(|target| target.submission_state.clone()),
        display_state: target.map_or("idle", |target| target.display_state),
        blocked_reason: target.and_then(|target| target.blocked_reason.clone()),
        tx_hash: target.and_then(|target| target.tx_hash.clone()),
        error_message: target.and_then(|target| target.error_message.clone()),
    }
}

fn replication_target(
    snapshot: &JobSnapshot,
    submission: ChainSubmission,
) -> ReplicationTargetResponse {
    let (display_state, blocked_reason) = target_display_state(snapshot, &submission);

    ReplicationTargetResponse {
        chain_name: submission.chain_name,
        chain_id: submission.chain_id,
        registry_address: submission.registry_address,
        submission_state: submission.submission_state.as_db_str().to_string(),
        tx_hash: submission.submission_tx_hash,
        error_message: submission.submission_error_message,
        retry_count: submission.submission_retry_count,
        display_state,
        blocked_reason,
    }
}

fn target_display_state(
    snapshot: &JobSnapshot,
    submission: &ChainSubmission,
) -> (&'static str, Option<String>) {
    match submission.submission_state {
        ChainSubmissionState::Confirmed => ("confirmed", None),
        ChainSubmissionState::Submitting => ("submitting", None),
        ChainSubmissionState::Failed => ("failed", None),
        ChainSubmissionState::Pending => pending_display_state(snapshot),
    }
}

fn pending_display_state(snapshot: &JobSnapshot) -> (&'static str, Option<String>) {
    match snapshot.job_state {
        ReplicationJobState::WaitingFinality => (
            "blocked",
            Some("Waiting for Bankai finality on the exact L1 source block".to_string()),
        ),
        ReplicationJobState::ReadyToProve | ReplicationJobState::ProofInProgress => (
            "blocked",
            Some("Generating the shared SP1 proof before fan-out can begin".to_string()),
        ),
        ReplicationJobState::ProofReady | ReplicationJobState::Submitting => ("queued", None),
        ReplicationJobState::Completed => ("confirmed", None),
        ReplicationJobState::Failed => {
            if snapshot.bankai_finalized_at.is_none() {
                (
                    "blocked",
                    Some(
                        "The job failed before Bankai finality cleared for this root update"
                            .to_string(),
                    ),
                )
            } else if snapshot.proof_artifact_ref.is_none() {
                (
                    "blocked",
                    Some("The job failed before the shared SP1 proof became available".to_string()),
                )
            } else {
                ("queued", None)
            }
        }
    }
}

fn stage(snapshot: &JobSnapshot) -> StageInfo {
    match snapshot.job_state {
        ReplicationJobState::WaitingFinality => StageInfo {
            label: "Waiting for Bankai finality",
            description:
                "The World ID root was submitted to Ethereum Sepolia, but the exact source block is not yet finalized in Bankai's finalized view."
                    .to_string(),
            blocked_by: Some("bankai_finality"),
        },
        ReplicationJobState::ReadyToProve => StageInfo {
            label: "Queued for proving",
            description:
                "The source block is finalized, and the replication job is queued to generate the shared SP1 proof."
                    .to_string(),
            blocked_by: Some("proving"),
        },
        ReplicationJobState::ProofInProgress => StageInfo {
            label: "Generating proof",
            description:
                "The backend is generating the shared SP1 proof that every destination chain depends on."
                    .to_string(),
            blocked_by: Some("proving"),
        },
        ReplicationJobState::ProofReady => StageInfo {
            label: "Ready to replicate",
            description:
                "The shared SP1 proof is ready, and destination submissions can begin."
                    .to_string(),
            blocked_by: None,
        },
        ReplicationJobState::Submitting => StageInfo {
            label: "Replicating to targets",
            description:
                "The shared proof is ready, and the backend is sending or confirming submissions across destination chains."
                    .to_string(),
            blocked_by: None,
        },
        ReplicationJobState::Completed => StageInfo {
            label: "Replication complete",
            description:
                "Every configured destination chain confirmed the latest replicated World ID root."
                    .to_string(),
            blocked_by: None,
        },
        ReplicationJobState::Failed => StageInfo {
            label: "Needs attention",
            description: snapshot.job_error_message.clone().unwrap_or_else(|| {
                "The latest replication job failed before every target settled successfully."
                    .to_string()
            }),
            blocked_by: None,
        },
    }
}

struct StageInfo {
    label: &'static str,
    description: String,
    blocked_by: Option<&'static str>,
}
