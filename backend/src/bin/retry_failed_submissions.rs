use anyhow::{bail, Context, Result};
use std::env;
use std::path::PathBuf;
use std::str::FromStr;
use world_id_root_replicator_backend::{
    db,
    jobs::types::{ChainSubmissionState, DestinationChain},
};

#[tokio::main]
async fn main() -> Result<()> {
    load_env();

    let args = Args::parse()?;
    let database_url = required("DATABASE_URL")?;
    let pool = db::connect(&database_url).await?;
    db::migrate(&pool).await?;

    let snapshot = db::job_snapshot(&pool, args.job_id)
        .await?
        .with_context(|| format!("job {} not found", args.job_id))?;
    let artifact_path = snapshot
        .proof_artifact_ref
        .as_deref()
        .with_context(|| format!("job {} has no proof artifact to retry", args.job_id))?;

    let submissions = db::job_submissions(&pool, args.job_id).await?;
    let selected = submissions
        .into_iter()
        .filter(|submission| submission.submission_state == ChainSubmissionState::Failed)
        .filter(|submission| {
            args.chain_name
                .as_deref()
                .is_none_or(|chain_name| submission.chain_name == chain_name)
        })
        .collect::<Vec<_>>();

    if selected.is_empty() {
        bail!("no failed submissions matched the requested filters");
    }

    for submission in &selected {
        db::reset_submission_for_retry(&pool, submission.submission_id).await?;
    }
    db::reset_job_for_retry(&pool, args.job_id).await?;

    println!("Reset job {} for retry.", args.job_id);
    println!("Proof artifact: {artifact_path}");
    for submission in &selected {
        println!(
            "Reset chain submission {} ({}) back to pending.",
            submission.submission_id, submission.chain_name
        );
    }
    println!("If the runner is already running, it will pick this up on the next loop.");

    Ok(())
}

struct Args {
    job_id: i64,
    chain_name: Option<String>,
}

impl Args {
    fn parse() -> Result<Self> {
        Self::parse_from(env::args())
    }

    fn parse_from(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut job_id = None;
        let mut chain_name = None;

        let mut args = args.into_iter().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--job" => {
                    let raw = next_value(&mut args, "--job")?;
                    job_id = Some(
                        raw.parse()
                            .with_context(|| format!("parse --job value `{raw}` as integer"))?,
                    );
                }
                "--chain" => {
                    let raw = next_value(&mut args, "--chain")?;
                    let chain = DestinationChain::from_str(&raw)?;
                    chain_name = Some(chain.as_str().to_string());
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                _ => bail!("unknown argument `{arg}`"),
            }
        }

        Ok(Self {
            job_id: job_id.context("missing required --job")?,
            chain_name,
        })
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .with_context(|| format!("missing value for {flag}"))
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run -p world-id-root-replicator-backend --bin retry_failed_submissions -- \\
  --job <job-id> [--chain <base-sepolia|op-sepolia|arbitrum-sepolia|starknet-sepolia|solana-devnet|chiado|monad-testnet|hyperevm-testnet|tempo-testnet|megaeth-testnet|plasma-testnet>]"
    );
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be set"))
}

fn load_env() {
    if dotenv::dotenv().is_ok() {
        return;
    }

    let workspace_env = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.env");
    dotenv::from_path(&workspace_env).ok();
}

#[cfg(test)]
mod tests {
    use super::Args;

    #[test]
    fn parses_job_and_chain_flags() {
        let args = Args::parse_from(
            [
                "retry_failed_submissions",
                "--job",
                "66",
                "--chain",
                "solana-devnet",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .unwrap();

        assert_eq!(args.job_id, 66);
        assert_eq!(args.chain_name.as_deref(), Some("solana-devnet"));
    }
}
