use anchor_lang::prelude::*;

pub mod state;

use state::{RegistryState, RootRecord};

declare_id!("CGPJkHwUYwubDNoaLwEMMNqHcHkKz3wB3SKb2ST4i2G1");

const ROOT_SEED: &[u8] = b"root";
const STATE_SEED: &[u8] = b"state";

#[program]
pub mod world_id_root_registry_solana {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, program_vkey_hash: [u8; 32]) -> Result<()> {
        let state = &mut ctx.accounts.state;
        state.program_vkey_hash = program_vkey_hash;
        state.latest_root = [0u8; 32];
        state.latest_source_block = 0;
        state.bump = ctx.bumps.state;
        Ok(())
    }

    pub fn submit_root(
        ctx: Context<SubmitRoot>,
        source_block_number: u64,
        sp1_public_inputs: Vec<u8>,
        groth16_proof: Vec<u8>,
    ) -> Result<()> {
        let state = &mut ctx.accounts.state;
        let root_record = &mut ctx.accounts.root_record;
        let decoded = decode_public_values(&sp1_public_inputs)?;

        require!(
            decoded.source_block_number == source_block_number,
            RootRegistryError::SourceBlockMismatch
        );
        require!(
            source_block_number > state.latest_source_block,
            RootRegistryError::StaleSourceBlock
        );

        let vkey_hash = format!("0x{}", to_hex(state.program_vkey_hash));
        sp1_solana::verify_proof(
            &groth16_proof,
            &sp1_public_inputs,
            vkey_hash.as_str(),
            sp1_solana::GROTH16_VK_5_0_0_BYTES,
        )
        .map_err(|_| error!(RootRegistryError::InvalidProof))?;

        apply_verified_root(
            state,
            root_record,
            source_block_number,
            decoded,
            ctx.bumps.root_record,
        )?;

        emit!(RootReplicated {
            source_block_number: decoded.source_block_number,
            root: decoded.root,
        });

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init,
        payer = payer,
        space = RegistryState::SPACE,
        seeds = [STATE_SEED],
        bump
    )]
    pub state: Account<'info, RegistryState>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
#[instruction(source_block_number: u64)]
pub struct SubmitRoot<'info> {
    #[account(mut, seeds = [STATE_SEED], bump = state.bump)]
    pub state: Account<'info, RegistryState>,
    #[account(
        init_if_needed,
        payer = payer,
        space = RootRecord::SPACE,
        seeds = [ROOT_SEED, &source_block_number.to_be_bytes()],
        bump
    )]
    pub root_record: Account<'info, RootRecord>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[event]
pub struct RootReplicated {
    pub source_block_number: u64,
    pub root: [u8; 32],
}

#[derive(Clone, Copy)]
struct DecodedPublicValues {
    source_block_number: u64,
    root: [u8; 32],
}

fn apply_verified_root(
    state: &mut RegistryState,
    root_record: &mut RootRecord,
    source_block_number: u64,
    decoded: DecodedPublicValues,
    root_record_bump: u8,
) -> Result<()> {
    require!(
        decoded.source_block_number == source_block_number,
        RootRegistryError::SourceBlockMismatch
    );
    require!(
        source_block_number > state.latest_source_block,
        RootRegistryError::StaleSourceBlock
    );

    if root_record.initialized && root_record.root != decoded.root {
        return err!(RootRegistryError::ConflictingRoot);
    }

    root_record.source_block_number = decoded.source_block_number;
    root_record.root = decoded.root;
    root_record.initialized = true;
    root_record.bump = root_record_bump;

    state.latest_source_block = decoded.source_block_number;
    state.latest_root = decoded.root;

    Ok(())
}

fn decode_public_values(bytes: &[u8]) -> Result<DecodedPublicValues> {
    require!(bytes.len() == 64, RootRegistryError::InvalidPublicInputs);

    let source_block_number = u64::from_be_bytes(
        bytes[24..32]
            .try_into()
            .map_err(|_| error!(RootRegistryError::InvalidPublicInputs))?,
    );
    let root = bytes[32..64]
        .try_into()
        .map_err(|_| error!(RootRegistryError::InvalidPublicInputs))?;

    Ok(DecodedPublicValues {
        source_block_number,
        root,
    })
}

fn to_hex(bytes: [u8; 32]) -> String {
    let mut encoded = String::with_capacity(64);
    for byte in bytes {
        use core::fmt::Write;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

#[error_code]
pub enum RootRegistryError {
    #[msg("Invalid SP1 proof")]
    InvalidProof,
    #[msg("Invalid public inputs layout")]
    InvalidPublicInputs,
    #[msg("Source block did not match the provided PDA seed")]
    SourceBlockMismatch,
    #[msg("Conflicting root for the same source block")]
    ConflictingRoot,
    #[msg("Source block is stale or out of order")]
    StaleSourceBlock,
}

#[cfg(test)]
mod tests {
    use super::{apply_verified_root, decode_public_values, to_hex, DecodedPublicValues};
    use crate::state::{RegistryState, RootRecord};

    #[test]
    fn decode_public_values_reads_abi_layout() {
        let mut bytes = [0u8; 64];
        bytes[24..32].copy_from_slice(&42u64.to_be_bytes());
        bytes[32..64].copy_from_slice(&[7u8; 32]);

        let decoded = decode_public_values(&bytes).unwrap();
        assert_eq!(decoded.source_block_number, 42);
        assert_eq!(decoded.root, [7u8; 32]);
    }

    #[test]
    fn to_hex_encodes_32_bytes() {
        assert_eq!(to_hex([0xab; 32]).len(), 64);
    }

    #[test]
    fn apply_verified_root_updates_state_and_record() {
        let mut state = RegistryState {
            program_vkey_hash: [0u8; 32],
            latest_root: [0u8; 32],
            latest_source_block: 0,
            bump: 1,
        };
        let mut root_record = RootRecord {
            source_block_number: 0,
            root: [0u8; 32],
            initialized: false,
            bump: 0,
        };

        apply_verified_root(
            &mut state,
            &mut root_record,
            42,
            DecodedPublicValues {
                source_block_number: 42,
                root: [7u8; 32],
            },
            9,
        )
        .unwrap();

        assert_eq!(state.latest_source_block, 42);
        assert_eq!(state.latest_root, [7u8; 32]);
        assert_eq!(root_record.source_block_number, 42);
        assert_eq!(root_record.root, [7u8; 32]);
        assert!(root_record.initialized);
        assert_eq!(root_record.bump, 9);
    }

    #[test]
    fn apply_verified_root_rejects_stale_blocks() {
        let mut state = RegistryState {
            program_vkey_hash: [0u8; 32],
            latest_root: [3u8; 32],
            latest_source_block: 42,
            bump: 1,
        };
        let mut root_record = RootRecord {
            source_block_number: 0,
            root: [0u8; 32],
            initialized: false,
            bump: 0,
        };

        let error = apply_verified_root(
            &mut state,
            &mut root_record,
            42,
            DecodedPublicValues {
                source_block_number: 42,
                root: [7u8; 32],
            },
            9,
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Source block is stale or out of order")
        );
    }

    #[test]
    fn apply_verified_root_rejects_conflicting_roots() {
        let mut state = RegistryState {
            program_vkey_hash: [0u8; 32],
            latest_root: [3u8; 32],
            latest_source_block: 99,
            bump: 1,
        };
        let mut root_record = RootRecord {
            source_block_number: 100,
            root: [8u8; 32],
            initialized: true,
            bump: 4,
        };

        let error = apply_verified_root(
            &mut state,
            &mut root_record,
            100,
            DecodedPublicValues {
                source_block_number: 100,
                root: [7u8; 32],
            },
            9,
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("Conflicting root for the same source block")
        );
    }
}
