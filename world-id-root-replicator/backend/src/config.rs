use crate::jobs::types::DestinationChain;
use alloy_primitives::Address;
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
    pub registry_address: Address,
    pub private_key: String,
}

impl DestinationChainConfig {
    pub const fn name(&self) -> &'static str {
        self.chain.as_str()
    }

    pub const fn chain_id(&self) -> u64 {
        self.chain.chain_id()
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
            ],
        })
    }
}

fn destination_chain_config(chain: DestinationChain) -> Result<DestinationChainConfig> {
    let prefix = chain.env_prefix();

    Ok(DestinationChainConfig {
        chain,
        rpc_url: required(&format!("{prefix}_RPC_URL"))?,
        registry_address: parse_address(&format!("{prefix}_REGISTRY_ADDRESS"))?,
        private_key: required(&format!("{prefix}_PRIVATE_KEY"))?,
    })
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be set"))
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

fn parse_address(name: &str) -> Result<Address> {
    required(name)?
        .parse()
        .with_context(|| format!("{name} must be a valid address"))
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
