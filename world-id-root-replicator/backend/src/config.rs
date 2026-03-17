use alloy_primitives::{Address, U256};
use anyhow::{Context, Result};
use bankai_sdk::Network;
use std::env;
use std::net::SocketAddr;

#[derive(Clone, Debug)]
pub struct Config {
    pub database_url: String,
    pub listen_addr: SocketAddr,
    pub bankai_network: BankaiNetwork,
    pub sp1_prover: String,
    pub execution_rpc: String,
    pub world_id_identity_manager: Address,
    pub world_id_root_slot: U256,
    pub base_sepolia: DestinationChainConfig,
}

#[derive(Clone, Debug)]
pub struct DestinationChainConfig {
    pub name: &'static str,
    pub chain_id: u64,
    pub rpc_url: String,
    pub registry_address: Address,
    pub private_key: String,
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
            sp1_prover: required("SP1_PROVER")?,
            execution_rpc: required("EXECUTION_RPC")?,
            world_id_identity_manager: parse_address("WORLD_ID_IDENTITY_MANAGER")?,
            world_id_root_slot: parse_u256("WORLD_ID_ROOT_SLOT")?,
            base_sepolia: DestinationChainConfig {
                name: "base-sepolia",
                chain_id: 84_532,
                rpc_url: required("BASE_SEPOLIA_RPC_URL")?,
                registry_address: parse_address("BASE_SEPOLIA_REGISTRY_ADDRESS")?,
                private_key: required("BASE_SEPOLIA_PRIVATE_KEY")?,
            },
        })
    }
}

fn required(name: &str) -> Result<String> {
    env::var(name).with_context(|| format!("{name} must be set"))
}

fn parse_address(name: &str) -> Result<Address> {
    required(name)?
        .parse()
        .with_context(|| format!("{name} must be a valid address"))
}

fn parse_u256(name: &str) -> Result<U256> {
    let raw = required(name)?;
    let trimmed = raw.trim_start_matches("0x");
    U256::from_str_radix(trimmed, 16).with_context(|| format!("{name} must be a hex uint256"))
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
