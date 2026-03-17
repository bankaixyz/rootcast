use anyhow::Result;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use world_id_root_replicator_backend::{api, config::Config, db, jobs::runner::Runner};

#[tokio::main]
async fn main() -> Result<()> {
    load_env();
    setup_tracing();

    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;

    let runner = Runner::from_config(config.clone(), pool.clone())?;
    tokio::spawn(async move {
        if let Err(error) = runner.run_forever().await {
            error!(?error, "replication runner stopped");
        }
    });

    let app = api::router(pool.clone(), config.destination_chains.clone());
    let listener = TcpListener::bind(config.listen_addr).await?;

    info!(
        listen_addr = %config.listen_addr,
        database_url = %config.database_url,
        "world-id-root-replicator backend initialized"
    );

    axum::serve(listener, app).await?;
    Ok(())
}

fn load_env() {
    if dotenv::dotenv().is_ok() {
        return;
    }

    let workspace_env = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.env");
    dotenv::from_path(&workspace_env).ok();
}

fn setup_tracing() {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        EnvFilter::new("world_id_root_replicator_backend=info,tower_http=info")
    });

    tracing_subscriber::fmt().with_env_filter(filter).init();
}
