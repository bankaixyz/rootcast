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
}

impl DestinationChain {
    pub const fn chain_id(self) -> u64 {
        match self {
            Self::BaseSepolia => 84_532,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BaseSepolia => "base-sepolia",
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
