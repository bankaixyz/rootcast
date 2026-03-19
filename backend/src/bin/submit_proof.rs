use anyhow::{bail, Context, Result};
use std::env;
use std::path::PathBuf;
use tokio::time::{sleep, timeout, Duration};
use world_id_root_replicator_backend::{
    chains::{EvmSubmitter, SolanaSubmitter, StarknetSubmitter, SubmissionCheck, SubmissionClient},
    config::{Config, DestinationChainConfig},
    jobs::types::DestinationChain,
    proving::sp1::{current_program_vkey, root_to_hex, ProofService, Sp1ProofService},
};

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const SUBMIT_TIMEOUT: Duration = Duration::from_secs(45);

#[tokio::main]
async fn main() -> Result<()> {
    load_env();

    let args = Args::parse()?;
    let config = Config::from_env()?;
    let destination = find_destination(&config.destination_chains, &args.chain)?;
    let registry_address = args
        .registry_address
        .clone()
        .unwrap_or_else(|| destination.contract_address.clone());

    let proof_service = Sp1ProofService::new(PathBuf::from("artifacts/proofs"), true);
    let artifact = proof_service
        .load(&args.artifact_path)
        .await
        .with_context(|| format!("load proof artifact from {}", args.artifact_path))?;

    println!("Chain: {}", destination.name());
    println!("Artifact: {}", args.artifact_path);
    println!("Registry: {}", registry_address);
    println!(
        "Decoded public values: source_block_number={} root={}",
        artifact.decoded_public_values.source_block_number,
        root_to_hex(artifact.decoded_public_values.root)
    );
    println!("Submitting proof artifact...");

    let client = submission_client(destination.clone())?;
    let tx_hash = timeout(
        SUBMIT_TIMEOUT,
        client.submit_artifact(&registry_address, &args.artifact_path),
    )
    .await
    .context("timed out waiting for submission client")??;

    println!("Submitted transaction: {tx_hash}");

    if args.wait {
        loop {
            match client.check_submission(&tx_hash).await? {
                SubmissionCheck::Pending => {
                    println!("Submission still pending, polling again in 5s...");
                    sleep(POLL_INTERVAL).await;
                }
                SubmissionCheck::Confirmed => {
                    println!("Submission confirmed: {tx_hash}");
                    break;
                }
                SubmissionCheck::Failed(message) => {
                    bail!("submission failed: {message}");
                }
            }
        }
    }

    Ok(())
}

struct Args {
    chain: String,
    artifact_path: String,
    registry_address: Option<String>,
    wait: bool,
}

impl Args {
    fn parse() -> Result<Self> {
        Self::parse_from(env::args())
    }

    fn parse_from(args: impl IntoIterator<Item = String>) -> Result<Self> {
        let mut chain = None;
        let mut artifact_path = None;
        let mut registry_address = None;
        let mut wait = false;

        let mut args = args.into_iter().skip(1);
        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--chain" => {
                    chain = Some(next_value(&mut args, "--chain")?);
                }
                "--artifact" => {
                    artifact_path = Some(next_value(&mut args, "--artifact")?);
                }
                "--registry" => {
                    registry_address = Some(next_value(&mut args, "--registry")?);
                }
                "--wait" => {
                    wait = true;
                }
                "--help" | "-h" => {
                    print_usage();
                    std::process::exit(0);
                }
                _ => bail!("unknown argument `{arg}`"),
            }
        }

        Ok(Self {
            chain: chain.context("missing required --chain")?,
            artifact_path: artifact_path.context("missing required --artifact")?,
            registry_address,
            wait,
        })
    }
}

fn next_value(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String> {
    args.next()
        .with_context(|| format!("missing value for {flag}"))
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run -p world-id-root-replicator-backend --bin submit_proof -- \\
  --chain <base-sepolia|op-sepolia|arbitrum-sepolia|starknet-sepolia|solana-devnet> \\
  --artifact <path> [--registry <address>] [--wait]"
    );
}

fn find_destination<'a>(
    destinations: &'a [DestinationChainConfig],
    chain: &str,
) -> Result<&'a DestinationChainConfig> {
    destinations
        .iter()
        .find(|destination| destination.name() == chain)
        .with_context(|| format!("unknown chain `{chain}`"))
}

fn submission_client(destination: DestinationChainConfig) -> Result<Box<dyn SubmissionClient>> {
    let client: Box<dyn SubmissionClient> = match destination.chain {
        DestinationChain::StarknetSepolia => {
            Box::new(StarknetSubmitter::new(destination, current_program_vkey()))
        }
        DestinationChain::SolanaDevnet => Box::new(SolanaSubmitter::new(destination)?),
        _ => Box::new(EvmSubmitter::new(destination)),
    };

    Ok(client)
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
    use super::{find_destination, Args};
    use world_id_root_replicator_backend::{
        config::DestinationChainConfig, jobs::types::DestinationChain,
    };

    #[test]
    fn finds_destination_by_chain_name() {
        let destinations = vec![DestinationChainConfig {
            chain: DestinationChain::SolanaDevnet,
            rpc_url: "https://example.invalid".to_string(),
            contract_address: "HpgNxwdekXixEW6ZzTPsjhhFx46fpfoC7ruJvsinPYHx".to_string(),
            private_key: "0x1".to_string(),
            account_address: None,
        }];

        let destination = find_destination(&destinations, "solana-devnet").unwrap();
        assert_eq!(destination.name(), "solana-devnet");
    }

    #[test]
    fn parses_wait_and_registry_flags() {
        let parsed = parse_args([
            "submit_proof",
            "--chain",
            "solana-devnet",
            "--artifact",
            "artifacts/proofs/job-1.bin",
            "--registry",
            "HpgNxwdekXixEW6ZzTPsjhhFx46fpfoC7ruJvsinPYHx",
            "--wait",
        ])
        .unwrap();

        assert_eq!(parsed.chain, "solana-devnet");
        assert_eq!(parsed.artifact_path, "artifacts/proofs/job-1.bin");
        assert_eq!(
            parsed.registry_address.as_deref(),
            Some("HpgNxwdekXixEW6ZzTPsjhhFx46fpfoC7ruJvsinPYHx")
        );
        assert!(parsed.wait);
    }

    fn parse_args<const N: usize>(args: [&str; N]) -> Result<Args, anyhow::Error> {
        Args::parse_from(args.into_iter().map(str::to_string))
    }
}
