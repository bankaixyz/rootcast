//! A zkVM program that verifies a Bankai proof bundle and commits the verified
//! World ID root plus the source block number as typed public output.

#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{address, uint, Address, FixedBytes, U256};
use alloy_sol_types::SolValue;
use bankai_types::ProofBundle;
use bankai_verify::verify_batch_proof;
use serde::{Deserialize, Serialize};

const SEPOLIA_IDENTITY_MANAGER: Address = address!("b2EaD588f14e69266d1b87936b75325181377076");
const LATEST_ROOT_SLOT: U256 = uint!(0x12e_U256);

#[derive(Debug, Deserialize, Serialize)]
pub struct PublicValues {
    pub source_block_number: u64,
    pub root: [u8; 32],
}

pub fn main() {
    let bundle = sp1_zkvm::io::read::<ProofBundle>();

    let result = verify_batch_proof(bundle).expect("failed to verify batch proof");
    assert_eq!(
        result.evm.storage_slot.len(),
        1,
        "expected exactly one verified storage-slot request"
    );

    let storage_slot = &result.evm.storage_slot[0];
    assert_eq!(
        storage_slot.slots.len(),
        1,
        "expected exactly one verified storage slot value"
    );
    assert_eq!(
        storage_slot.address, SEPOLIA_IDENTITY_MANAGER,
        "expected proof for the Sepolia World ID Identity Manager"
    );
    assert_eq!(
        storage_slot.slots[0].0, LATEST_ROOT_SLOT,
        "expected proof for the World ID latest-root storage slot"
    );

    let public_values = PublicValues {
        source_block_number: storage_slot.block.block_number,
        root: storage_slot.slots[0].1.to_be_bytes(),
    };

    let encoded_public_values = (
        public_values.source_block_number,
        FixedBytes::<32>::from(public_values.root),
    )
        .abi_encode();

    sp1_zkvm::io::commit_slice(&encoded_public_values);
}
