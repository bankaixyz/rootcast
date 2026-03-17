use alloy_primitives::FixedBytes;
use alloy_sol_types::SolValue;
use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use bankai_types::inputs::ProofBundle;
use serde::{Deserialize, Serialize};
use sp1_sdk::{include_elf, ProverClient, SP1ProofWithPublicValues, SP1Stdin};
use std::fs;
use std::path::{Path, PathBuf};

const WORLD_ID_ROOT_REPLICATOR_ELF: &[u8] = include_elf!("world-id-root-replicator-program");

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct PublicValues {
    pub source_block_number: u64,
    pub root: [u8; 32],
}

#[derive(Clone, Debug)]
pub struct ProofArtifact {
    pub path: String,
    pub public_values: Vec<u8>,
    pub decoded_public_values: PublicValues,
}

#[async_trait]
pub trait ProofService: Send + Sync {
    async fn prove(
        &self,
        job_id: i64,
        bundle_bytes: &[u8],
        expected_public_values: &PublicValues,
    ) -> Result<ProofArtifact>;

    async fn load(&self, artifact_path: &str) -> Result<ProofArtifact>;
}

pub struct Sp1ProofService {
    artifact_dir: PathBuf,
}

impl Sp1ProofService {
    pub fn new(artifact_dir: impl Into<PathBuf>) -> Self {
        Self {
            artifact_dir: artifact_dir.into(),
        }
    }

    fn artifact_path(&self, job_id: i64) -> PathBuf {
        self.artifact_dir.join(format!("job-{job_id}.bin"))
    }
}

#[async_trait]
impl ProofService for Sp1ProofService {
    async fn prove(
        &self,
        job_id: i64,
        bundle_bytes: &[u8],
        expected_public_values: &PublicValues,
    ) -> Result<ProofArtifact> {
        fs::create_dir_all(&self.artifact_dir).with_context(|| {
            format!(
                "create proof artifact directory {}",
                self.artifact_dir.display()
            )
        })?;

        let bundle: ProofBundle =
            bincode::deserialize(bundle_bytes).context("deserialize Bankai proof bundle")?;

        let mut stdin = SP1Stdin::new();
        stdin.write(&bundle);

        let client = ProverClient::from_env();
        let (pk, vk) = client.setup(WORLD_ID_ROOT_REPLICATOR_ELF);
        let proof = client
            .prove(&pk, &stdin)
            .groth16()
            .run()
            .context("generate SP1 Groth16 proof")?;

        client
            .verify(&proof, &vk)
            .context("verify generated SP1 proof")?;

        let public_values = proof.public_values.to_vec();
        let decoded_public_values = decode_public_values(&public_values)?;
        if &decoded_public_values != expected_public_values {
            return Err(anyhow!(
                "decoded proof public values do not match observed root: expected block {} root {}, got block {} root {}",
                expected_public_values.source_block_number,
                root_to_hex(expected_public_values.root),
                decoded_public_values.source_block_number,
                root_to_hex(decoded_public_values.root),
            ));
        }

        let artifact_path = self.artifact_path(job_id);
        proof
            .save(&artifact_path)
            .with_context(|| format!("save SP1 proof artifact to {}", artifact_path.display()))?;

        Ok(ProofArtifact {
            path: artifact_path.display().to_string(),
            public_values,
            decoded_public_values,
        })
    }

    async fn load(&self, artifact_path: &str) -> Result<ProofArtifact> {
        let proof = load_proof(artifact_path)?;
        let public_values = proof.public_values.to_vec();
        let decoded_public_values = decode_public_values(&public_values)?;

        Ok(ProofArtifact {
            path: artifact_path.to_string(),
            public_values,
            decoded_public_values,
        })
    }
}

pub fn load_proof(path: impl AsRef<Path>) -> Result<SP1ProofWithPublicValues> {
    SP1ProofWithPublicValues::load(path.as_ref())
        .with_context(|| format!("load SP1 proof artifact from {}", path.as_ref().display()))
}

pub fn decode_public_values(bytes: &[u8]) -> Result<PublicValues> {
    let (source_block_number, root): (u64, FixedBytes<32>) =
        <(u64, FixedBytes<32>) as SolValue>::abi_decode(bytes)
            .context("decode ABI-encoded public values")?;

    Ok(PublicValues {
        source_block_number,
        root: root.into(),
    })
}

pub fn root_to_hex(root: [u8; 32]) -> String {
    format!("0x{}", hex::encode(root))
}

pub fn root_hex_to_bytes(root_hex: &str) -> Result<[u8; 32]> {
    let trimmed = root_hex.trim_start_matches("0x");
    let bytes = hex::decode(trimmed).context("decode root hex")?;
    if bytes.len() != 32 {
        return Err(anyhow!("expected 32-byte root, got {} bytes", bytes.len()));
    }

    let mut root = [0u8; 32];
    root.copy_from_slice(&bytes);
    Ok(root)
}
