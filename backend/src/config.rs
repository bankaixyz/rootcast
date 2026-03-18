use crate::jobs::types::DestinationChain;
use anyhow::{Context, Result};
use bankai_sdk::Network;
use std::env;
use std::net::SocketAddr;
use std::str::FromStr;

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
            destination_chains: enabled_destination_chains()?
                .into_iter()
                .map(destination_chain_config)
                .collect::<Result<Vec<_>>>()?,
        })
    }
}

fn enabled_destination_chains() -> Result<Vec<DestinationChain>> {
    match env::var("ENABLED_DESTINATION_CHAINS") {
        Ok(value) => parse_enabled_destination_chains(&value),
        Err(env::VarError::NotPresent) => Ok(supported_destination_chains().to_vec()),
        Err(error) => Err(error).context("failed to read ENABLED_DESTINATION_CHAINS"),
    }
}

fn supported_destination_chains() -> &'static [DestinationChain] {
    &[
        DestinationChain::BaseSepolia,
        DestinationChain::OpSepolia,
        DestinationChain::ArbitrumSepolia,
        DestinationChain::StarknetSepolia,
        DestinationChain::SolanaDevnet,
        DestinationChain::Chiado,
        DestinationChain::MonadTestnet,
        DestinationChain::HyperEvmTestnet,
        DestinationChain::TempoTestnet,
        DestinationChain::MegaEthTestnet,
        DestinationChain::PlasmaTestnet,
    ]
}

fn parse_enabled_destination_chains(value: &str) -> Result<Vec<DestinationChain>> {
    let mut chains = Vec::new();

    for raw in value.split(',') {
        let name = raw.trim();
        if name.is_empty() {
            continue;
        }

        let chain = DestinationChain::from_str(name).with_context(|| {
            format!("unknown destination chain `{name}` in ENABLED_DESTINATION_CHAINS")
        })?;

        if !chains.contains(&chain) {
            chains.push(chain);
        }
    }

    if chains.is_empty() {
        anyhow::bail!("ENABLED_DESTINATION_CHAINS must include at least one supported chain name");
    }

    Ok(chains)
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

impl FromStr for DestinationChain {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "base" | "base-sepolia" => Ok(Self::BaseSepolia),
            "op" | "op-sepolia" | "optimism" | "optimism-sepolia" => Ok(Self::OpSepolia),
            "arb" | "arbitrum" | "arb-sepolia" | "arbitrum-sepolia" => Ok(Self::ArbitrumSepolia),
            "starknet" | "starknet-sepolia" => Ok(Self::StarknetSepolia),
            "solana" | "solana-devnet" => Ok(Self::SolanaDevnet),
            "chiado" | "gnosis" | "gnosis-chiado" => Ok(Self::Chiado),
            "monad" | "monad-testnet" => Ok(Self::MonadTestnet),
            "hyper" | "hyperevm" | "hyperevm-testnet" | "hyperliquid" => Ok(Self::HyperEvmTestnet),
            "tempo" | "tempo-testnet" => Ok(Self::TempoTestnet),
            "megaeth" | "megaeth-testnet" => Ok(Self::MegaEthTestnet),
            "plasma" | "plasma-testnet" => Ok(Self::PlasmaTestnet),
            _ => anyhow::bail!("unsupported destination chain: {value}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{parse_enabled_destination_chains, DestinationChain};

    #[test]
    fn parse_enabled_destination_chains_accepts_exact_names_and_aliases() {
        let chains =
            parse_enabled_destination_chains("base-sepolia, op, monad, hyperevm, plasma").unwrap();

        assert_eq!(
            chains,
            vec![
                DestinationChain::BaseSepolia,
                DestinationChain::OpSepolia,
                DestinationChain::MonadTestnet,
                DestinationChain::HyperEvmTestnet,
                DestinationChain::PlasmaTestnet,
            ]
        );
    }

    #[test]
    fn parse_enabled_destination_chains_deduplicates_and_skips_empty_entries() {
        let chains = parse_enabled_destination_chains("monad,, monad-testnet, chiado").unwrap();

        assert_eq!(
            chains,
            vec![DestinationChain::MonadTestnet, DestinationChain::Chiado]
        );
    }

    #[test]
    fn parse_enabled_destination_chains_rejects_unknown_names() {
        let error =
            parse_enabled_destination_chains("base-sepolia, definitely-not-a-chain").unwrap_err();

        assert!(error
            .to_string()
            .contains("unknown destination chain `definitely-not-a-chain`"));
    }

    #[test]
    fn parse_enabled_destination_chains_rejects_empty_selection() {
        let error = parse_enabled_destination_chains(" , ").unwrap_err();

        assert!(error
            .to_string()
            .contains("ENABLED_DESTINATION_CHAINS must include at least one supported chain"));
    }
}
