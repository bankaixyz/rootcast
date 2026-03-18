use anyhow::{Context, Result};
use std::env;
use std::path::PathBuf;
use world_id_root_replicator_backend::chains::solana::{initialize_registry, state_pda};

fn main() -> Result<()> {
    load_env();

    let command = env::args().nth(1).unwrap_or_else(|| "help".to_string());

    match command.as_str() {
        "initialize" => initialize(),
        "inspect" => inspect(),
        _ => {
            eprintln!("Usage: cargo run --bin solana_registry_admin -- [initialize|inspect]");
            Ok(())
        }
    }
}

fn initialize() -> Result<()> {
    let signature = initialize_registry(
        &required("SOLANA_DEVNET_RPC_URL")?,
        &required("SOLANA_DEVNET_PRIVATE_KEY")?,
        &required("SOLANA_DEVNET_PROGRAM_ID")?,
        &required("PROGRAM_VKEY")?,
    )?;

    println!("{signature}");
    Ok(())
}

fn inspect() -> Result<()> {
    let program_id = required("SOLANA_DEVNET_PROGRAM_ID")?;
    let program_id = program_id
        .parse()
        .with_context(|| format!("parse SOLANA_DEVNET_PROGRAM_ID `{program_id}`"))?;
    let (state_pda, _) = state_pda(&program_id);

    println!("program_id={program_id}");
    println!("state_pda={state_pda}");
    Ok(())
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
