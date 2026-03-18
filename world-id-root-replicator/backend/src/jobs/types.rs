use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ObservedRoot {
    pub root_hex: String,
    pub source_block_number: u64,
    pub source_tx_hash: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ReplicationJobState {
    WaitingFinality,
    ReadyToProve,
    ProofInProgress,
    ProofReady,
    Submitting,
    Completed,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ChainSubmissionState {
    Pending,
    Submitting,
    Confirmed,
    Failed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum DestinationChain {
    BaseSepolia,
    OpSepolia,
    ArbitrumSepolia,
    StarknetSepolia,
    SolanaDevnet,
    Chiado,
    MonadTestnet,
    HyperEvmTestnet,
    TempoTestnet,
    MegaEthTestnet,
    PlasmaTestnet,
}

impl DestinationChain {
    pub const fn chain_id(self) -> &'static str {
        match self {
            Self::BaseSepolia => "84532",
            Self::OpSepolia => "11155420",
            Self::ArbitrumSepolia => "421614",
            Self::StarknetSepolia => "0x534e5f5345504f4c4941",
            Self::SolanaDevnet => "devnet",
            Self::Chiado => "10200",
            Self::MonadTestnet => "10143",
            Self::HyperEvmTestnet => "998",
            Self::TempoTestnet => "42431",
            Self::MegaEthTestnet => "6343",
            Self::PlasmaTestnet => "9746",
        }
    }

    pub const fn evm_chain_id(self) -> Option<u64> {
        match self {
            Self::BaseSepolia => Some(84_532),
            Self::OpSepolia => Some(11_155_420),
            Self::ArbitrumSepolia => Some(421_614),
            Self::Chiado => Some(10_200),
            Self::MonadTestnet => Some(10_143),
            Self::HyperEvmTestnet => Some(998),
            Self::TempoTestnet => Some(42_431),
            Self::MegaEthTestnet => Some(6_343),
            Self::PlasmaTestnet => Some(9_746),
            Self::StarknetSepolia | Self::SolanaDevnet => None,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BaseSepolia => "base-sepolia",
            Self::OpSepolia => "op-sepolia",
            Self::ArbitrumSepolia => "arbitrum-sepolia",
            Self::StarknetSepolia => "starknet-sepolia",
            Self::SolanaDevnet => "solana-devnet",
            Self::Chiado => "chiado",
            Self::MonadTestnet => "monad-testnet",
            Self::HyperEvmTestnet => "hyperevm-testnet",
            Self::TempoTestnet => "tempo-testnet",
            Self::MegaEthTestnet => "megaeth-testnet",
            Self::PlasmaTestnet => "plasma-testnet",
        }
    }

    pub const fn env_prefix(self) -> &'static str {
        match self {
            Self::BaseSepolia => "BASE_SEPOLIA",
            Self::OpSepolia => "OP_SEPOLIA",
            Self::ArbitrumSepolia => "ARBITRUM_SEPOLIA",
            Self::StarknetSepolia => "STARKNET_SEPOLIA",
            Self::SolanaDevnet => "SOLANA_DEVNET",
            Self::Chiado => "CHIADO",
            Self::MonadTestnet => "MONAD_TESTNET",
            Self::HyperEvmTestnet => "HYPEREVM_TESTNET",
            Self::TempoTestnet => "TEMPO_TESTNET",
            Self::MegaEthTestnet => "MEGAETH_TESTNET",
            Self::PlasmaTestnet => "PLASMA_TESTNET",
        }
    }

    pub const fn is_evm(self) -> bool {
        match self {
            Self::BaseSepolia
            | Self::OpSepolia
            | Self::ArbitrumSepolia
            | Self::Chiado
            | Self::MonadTestnet
            | Self::HyperEvmTestnet
            | Self::TempoTestnet
            | Self::MegaEthTestnet
            | Self::PlasmaTestnet => true,
            Self::StarknetSepolia | Self::SolanaDevnet => false,
        }
    }
}

impl ReplicationJobState {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::WaitingFinality => "waiting_finality",
            Self::ReadyToProve => "ready_to_prove",
            Self::ProofInProgress => "proof_in_progress",
            Self::ProofReady => "proof_ready",
            Self::Submitting => "submitting",
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

impl ChainSubmissionState {
    pub const fn as_db_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Submitting => "submitting",
            Self::Confirmed => "confirmed",
            Self::Failed => "failed",
        }
    }
}

impl fmt::Display for ReplicationJobState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

impl fmt::Display for ChainSubmissionState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_db_str())
    }
}

impl FromStr for ReplicationJobState {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "waiting_finality" => Ok(Self::WaitingFinality),
            "ready_to_prove" => Ok(Self::ReadyToProve),
            "proof_in_progress" => Ok(Self::ProofInProgress),
            "proof_ready" => Ok(Self::ProofReady),
            "submitting" => Ok(Self::Submitting),
            "completed" => Ok(Self::Completed),
            "failed" => Ok(Self::Failed),
            _ => Err(anyhow!("unknown replication job state: {value}")),
        }
    }
}

impl FromStr for ChainSubmissionState {
    type Err = anyhow::Error;

    fn from_str(value: &str) -> Result<Self> {
        match value {
            "pending" => Ok(Self::Pending),
            "submitting" => Ok(Self::Submitting),
            "confirmed" => Ok(Self::Confirmed),
            "failed" => Ok(Self::Failed),
            _ => Err(anyhow!("unknown chain submission state: {value}")),
        }
    }
}
