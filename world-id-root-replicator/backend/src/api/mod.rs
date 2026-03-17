pub mod read_models;

use crate::config::DestinationChainConfig;
use crate::db;
use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use read_models::{
    chain_status, root_snapshot, status_response, ChainsResponse, ConfiguredChain,
    JobDetailResponse, LatestRootResponse, RootsResponse, StatusResponse,
};
use serde_json::json;
use sqlx::SqlitePool;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    configured_chains: Vec<ConfiguredChain>,
}

pub fn router(pool: SqlitePool, destination_chains: Vec<DestinationChainConfig>) -> Router {
    let configured_chains = destination_chains
        .into_iter()
        .map(|chain| ConfiguredChain {
            name: chain.name(),
            chain_id: chain.chain_id().to_string(),
            registry_address: chain.contract_address,
        })
        .collect();

    Router::new()
        .route("/api/status", get(status))
        .route("/api/roots/latest", get(latest_root))
        .route("/api/roots", get(roots))
        .route("/api/chains", get(chains))
        .route("/api/jobs/:id", get(job_detail))
        .with_state(AppState {
            pool,
            configured_chains,
        })
}

async fn status(State(state): State<AppState>) -> ApiResult<Json<StatusResponse>> {
    let latest_observed_source_block = db::latest_observed_source_block(&state.pool).await?;
    let latest_proof_request_age_seconds = db::latest_proof_request(&state.pool)
        .await?
        .map(|request| request.age_seconds);
    let latest_snapshot = db::latest_job_snapshot(&state.pool).await?;

    Ok(Json(status_response(
        state.configured_chains.len(),
        latest_observed_source_block,
        latest_proof_request_age_seconds,
        latest_snapshot.as_ref(),
    )))
}

async fn latest_root(State(state): State<AppState>) -> ApiResult<Json<LatestRootResponse>> {
    let snapshot = load_latest_snapshot(&state.pool).await?;
    Ok(Json(LatestRootResponse { snapshot }))
}

async fn roots(State(state): State<AppState>) -> ApiResult<Json<RootsResponse>> {
    let snapshots = db::recent_job_snapshots(&state.pool, 10).await?;
    let mut roots = Vec::with_capacity(snapshots.len());

    for snapshot in snapshots {
        let submissions = db::job_submissions(&state.pool, snapshot.job_id).await?;
        roots.push(root_snapshot(snapshot, submissions));
    }

    Ok(Json(RootsResponse { roots }))
}

async fn chains(State(state): State<AppState>) -> ApiResult<Json<ChainsResponse>> {
    let latest_snapshot = load_latest_snapshot(&state.pool).await?;
    let chains = state
        .configured_chains
        .iter()
        .map(|chain| chain_status(chain, latest_snapshot.as_ref()))
        .collect();

    Ok(Json(ChainsResponse { chains }))
}

async fn job_detail(
    State(state): State<AppState>,
    Path(job_id): Path<i64>,
) -> ApiResult<Json<JobDetailResponse>> {
    let snapshot = db::job_snapshot(&state.pool, job_id)
        .await?
        .ok_or_else(|| ApiError::not_found(format!("job {job_id} was not found")))?;
    let submissions = db::job_submissions(&state.pool, snapshot.job_id).await?;

    Ok(Json(JobDetailResponse {
        job: root_snapshot(snapshot, submissions),
    }))
}

async fn load_latest_snapshot(
    pool: &SqlitePool,
) -> Result<Option<read_models::RootSnapshotResponse>> {
    let Some(snapshot) = db::latest_job_snapshot(pool).await? else {
        return Ok(None);
    };

    let submissions = db::job_submissions(pool, snapshot.job_id).await?;
    Ok(Some(root_snapshot(snapshot, submissions)))
}

type ApiResult<T> = std::result::Result<T, ApiError>;

struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found(message: String) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message,
        }
    }
}

impl From<anyhow::Error> for ApiError {
    fn from(error: anyhow::Error) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: error.to_string(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DestinationChainConfig;
    use crate::jobs::types::{DestinationChain, ObservedRoot, ReplicationJobState};
    use axum::body::{to_bytes, Body};
    use axum::http::Request;
    use serde_json::Value;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use tower::util::ServiceExt;

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    #[tokio::test]
    async fn latest_root_blocks_targets_while_waiting_for_finality() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        record_root(&pool, &destinations, [7u8; 32], 12_345, "0xfeed").await;

        let app = router(pool, destinations);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/roots/latest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let snapshot = &json["snapshot"];

        assert_eq!(snapshot["blocked_by"], "bankai_finality");
        assert_eq!(snapshot["source_tx_hash"], "0xfeed");
        assert_eq!(snapshot["proof_ready"], false);
        assert_eq!(snapshot["replication_triggered"], false);

        let targets = snapshot["targets"].as_array().unwrap();
        assert_eq!(targets.len(), 4);
        assert!(targets
            .iter()
            .all(|target| target["display_state"] == "blocked"));
    }

    #[tokio::test]
    async fn latest_root_reports_proof_and_fanout_progress() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        record_root(&pool, &destinations, [5u8; 32], 33_333, "0xcafe").await;

        db::mark_observed_root_finalized(&pool, 1, 12_345)
            .await
            .unwrap();
        db::mark_job_proof_ready(&pool, 1, "/tmp/proof.bin")
            .await
            .unwrap();
        db::mark_submission_submitting(
            &pool,
            1,
            "0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        )
        .await
        .unwrap();

        let app = router(pool, destinations);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/roots/latest")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let snapshot = &json["snapshot"];

        assert_eq!(snapshot["proof_ready"], true);
        assert_eq!(snapshot["replication_triggered"], true);
        assert_eq!(snapshot["bankai_finalized_block_number"], 12_345);
    }

    #[tokio::test]
    async fn chains_endpoint_preserves_mixed_target_outcomes() {
        let pool = test_pool().await;
        let destinations = all_destinations();
        record_root(&pool, &destinations, [9u8; 32], 22_222, "0xbeef").await;

        db::mark_observed_root_finalized(&pool, 1, 22_222)
            .await
            .unwrap();
        db::mark_job_proof_ready(&pool, 1, "/tmp/proof.bin")
            .await
            .unwrap();
        db::update_job_state(&pool, 1, ReplicationJobState::Submitting)
            .await
            .unwrap();
        db::mark_submission_confirmed(
            &pool,
            1,
            "0x1111111111111111111111111111111111111111111111111111111111111111",
        )
        .await
        .unwrap();
        db::mark_submission_failed(&pool, 2, "op revert")
            .await
            .unwrap();
        db::mark_submission_submitting(
            &pool,
            3,
            "0x3333333333333333333333333333333333333333333333333333333333333333",
        )
        .await
        .unwrap();

        let app = router(pool, destinations);
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/chains")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();
        let chains = json["chains"].as_array().unwrap();

        assert_eq!(chains[0]["display_state"], "confirmed");
        assert_eq!(chains[1]["display_state"], "failed");
        assert_eq!(chains[1]["error_message"], "op revert");
        assert_eq!(chains[2]["display_state"], "submitting");
    }

    async fn test_pool() -> SqlitePool {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .to_string();
        let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "world-id-root-replicator-api-{unique}-{counter}.db"
        ));
        let database_url = format!("sqlite://{}", path.display());
        let pool = db::connect(&database_url).await.unwrap();
        db::migrate(&pool).await.unwrap();
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
        ]
    }

    fn destination(chain: DestinationChain) -> DestinationChainConfig {
        let suffix = match chain {
            DestinationChain::BaseSepolia => "0123",
            DestinationChain::OpSepolia => "0456",
            DestinationChain::ArbitrumSepolia => "0789",
            DestinationChain::StarknetSepolia => "0abc",
        };

        DestinationChainConfig {
            chain,
            rpc_url: "https://example.invalid".to_string(),
            contract_address: format!("0x000000000000000000000000000000000000{suffix}"),
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
}
