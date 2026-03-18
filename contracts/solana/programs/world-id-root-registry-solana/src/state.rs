use anchor_lang::prelude::*;

#[account]
pub struct RegistryState {
    pub program_vkey_hash: [u8; 32],
    pub latest_root: [u8; 32],
    pub latest_source_block: u64,
    pub bump: u8,
}

impl RegistryState {
    pub const SPACE: usize = 8 + 32 + 32 + 8 + 1;
}

#[account]
pub struct RootRecord {
    pub source_block_number: u64,
    pub root: [u8; 32],
    pub initialized: bool,
    pub bump: u8,
}

impl RootRecord {
    pub const SPACE: usize = 8 + 8 + 32 + 1 + 1;
}
