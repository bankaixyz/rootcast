use crate::jobs::types::DestinationChain;
use anyhow::{Context, Result};
use bankai_sdk::Network;
use std::env;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub listen_addr: SocketAddr,
    pub bankai_network: BankaiNetwork,
    pub enforce_min_proof_request_gap: bool,
    pub sp1_prover: String,
    pub execution_rpc: String,
    pub destination_chains: Vec<DestinationChainConfig>,
}

#[derive(Clone, Debug)]
pub struct DestinationChainConfig {
    pub chain: DestinationChain,
    pub rpc_url: String,
    pub contract_address: String,
    pub private_key: String,
    pub account_address: Option<String>,
}

impl DestinationChainConfig {
    pub const fn name(&self) -> &'static str {
        self.chain.as_str()
    }

    pub const fn chain_id(&self) -> &'static str {
        self.chain.chain_id()
    }

    pub const fn is_evm(&self) -> bool {
        self.chain.is_evm()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BankaiNetwork {
    Sepolia,
    Local,
}

impl BankaiNetwork {
    pub const fn into_sdk(self) -> Network {
        match self {
            Self::Sepolia => Network::Sepolia,
            Self::Local => Network::Local,
        }
    }
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            database_url: required("DATABASE_URL")?,
            listen_addr: required("LISTEN_ADDR")?
                .parse()
                .context("LISTEN_ADDR must be a valid socket address")?,
            bankai_network: required("BANKAI_NETWORK")?.parse()?,
            enforce_min_proof_request_gap: optional_bool("ENFORCE_MIN_PROOF_REQUEST_GAP")?
                .unwrap_or(false),
            sp1_prover: required("SP1_PROVER")?,
            execution_rpc: required("EXECUTION_RPC")?,
            destination_chains: vec![
                destination_chain_config(DestinationChain::BaseSepolia)?,
                destination_chain_config(DestinationChain::OpSepolia)?,
                destination_chain_config(DestinationChain::ArbitrumSepolia)?,
                destination_chain_config(DestinationChain::StarknetSepolia)?,
                destination_chain_config(DestinationChain::SolanaDevnet)?,
            ],
        })
    }
}

fn destination_chain_config(chain: DestinationChain) -> Result<DestinationChainConfig> {
    if chain.is_evm() {
        let prefix = chain.env_prefix();
        return Ok(DestinationChainConfig {
            chain,
            rpc_url: required(&format!("{prefix}_RPC_URL"))?,
            contract_address: required(&format!("{prefix}_REGISTRY_ADDRESS"))?,
            private_key: required(&format!("{prefix}_PRIVATE_KEY"))?,
            account_address: None,
        });
    }

    if chain == DestinationChain::StarknetSepolia {
        return Ok(DestinationChainConfig {
            chain,
            rpc_url: required_any(&["STARKNET_SEPOLIA_RPC_URL", "STARKNET_SEPOLIA_RPC"])?,
            contract_address: required("STARKNET_SEPOLIA_REGISTRY_ADDRESS")?,
            private_key: required_any(&["STARKNET_SEPOLIA_PRIVATE_KEY", "STARKNET_PRIVATE_KEY"])?,
            account_address: Some(required_any(&[
                "STARKNET_SEPOLIA_ACCOUNT_ADDRESS",
                "STARKNET_ACCOUNT_ADDRESS",
            ])?),
        });
    }

    Ok(DestinationChainConfig {
        chain,
        rpc_url: required_any(&["SOLANA_DEVNET_RPC_URL", "SOLANA_DEVNET_RPC"])?,
        contract_address: required("SOLANA_DEVNET_PROGRAM_ID")?,
        private_key: required("SOLANA_DEVNET_PRIVATE_KEY")?,
        account_address: None,
    })
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be set"))
}

fn required_any(names: &[&str]) -> Result<String> {
    for name in names {
        if let Ok(value) = env::var(name) {
            return Ok(value);
        }
    }

    anyhow::bail!("one of {} must be set", names.join(", "))
}

fn optional_bool(name: &str) -> Result<Option<bool>> {
    match env::var(name) {
        Ok(value) => value
            .parse()
            .map(Some)
            .with_context(|| format!("{name} must be `true` or `false`")),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {name}")),
    }
}

impl std::str::FromStr for BankaiNetwork {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "sepolia" => Ok(Self::Sepolia),
            "local" => Ok(Self::Local),
            _ => anyhow::bail!("BANKAI_NETWORK must be `sepolia` or `local`"),
        }
    }
}
