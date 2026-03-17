use axum::{routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router {
    Router::new().route("/api/status", get(status))
}

#[derive(Serialize)]
struct StatusResponse {
    phase: &'static str,
    service: &'static str,
    status: &'static str,
}

async fn status() -> Json<StatusResponse> {
    Json(StatusResponse {
        phase: "phase-2-proving-slice",
        service: "world-id-root-replicator-backend",
        status: "ok",
    })
}
