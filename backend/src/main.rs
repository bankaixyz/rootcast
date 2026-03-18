use anyhow::Result;
use std::env;
use std::path::PathBuf;
use tokio::net::TcpListener;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;
use world_id_root_replicator_backend::{api, config::Config, db, jobs::runner::Runner};

#[tokio::main]
async fn main() -> Result<()> {
    load_env();
    setup_tracing();

    let args = Args::parse()?;
    let config = Config::from_env()?;
    let pool = db::connect(&config.database_url).await?;
    db::migrate(&pool).await?;

    if args.runner_only {
        let runner = Runner::from_config(config.clone(), pool.clone())?;
        info!(
            database_url = %config.database_url,
            "world-id-root-replicator backend running in runner-only mode"
        );
        return runner.run_forever().await;
    }

    if !args.api_only {
        let runner = Runner::from_config(config.clone(), pool.clone())?;
        tokio::spawn(async move {
            if let Err(error) = runner.run_forever().await {
                error!(?error, "replication runner stopped");
            }
        });
    }

    let app = api::router(pool.clone(), config.destination_chains.clone());
    let listener = TcpListener::bind(config.listen_addr).await?;

    if args.api_only {
        info!(
            listen_addr = %config.listen_addr,
            database_url = %config.database_url,
            "world-id-root-replicator backend running in api-only mode"
        );
    } else {
        info!(
            listen_addr = %config.listen_addr,
            database_url = %config.database_url,
            "world-id-root-replicator backend initialized"
        );
    }

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

#[derive(Debug)]
struct Args {
    api_only: bool,
    runner_only: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        Self::parse_from(env::args())
    }

    fn parse_from(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut api_only = false;
        let mut runner_only = false;

        for arg in args.into_iter().skip(1) {
            match arg.as_str() {
                "--api-only" => {
                    api_only = true;
                }
                "--runner-only" => {
                    runner_only = true;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                _ => anyhow::bail!("unknown argument `{arg}`"),
            }
        }

        if api_only && runner_only {
            anyhow::bail!("--api-only and --runner-only cannot be used together");
        }

        Ok(Self {
            api_only,
            runner_only,
        })
    }
}

fn print_usage() {
    eprintln!("Usage: world-id-root-replicator-backend [--api-only | --runner-only]");
}

#[cfg(test)]
mod tests {
    use super::Args;

    #[test]
    fn parses_runner_only_flag() {
        let args =
            Args::parse_from(["backend", "--runner-only"].into_iter().map(str::to_string)).unwrap();

        assert!(args.runner_only);
        assert!(!args.api_only);
    }

    #[test]
    fn parses_api_only_flag() {
        let args =
            Args::parse_from(["backend", "--api-only"].into_iter().map(str::to_string)).unwrap();

        assert!(args.api_only);
        assert!(!args.runner_only);
    }

    #[test]
    fn rejects_conflicting_modes() {
        let error = Args::parse_from(
            ["backend", "--api-only", "--runner-only"]
                .into_iter()
                .map(str::to_string),
        )
        .unwrap_err();

        assert_eq!(
            error.to_string(),
            "--api-only and --runner-only cannot be used together"
        );
    }
}
