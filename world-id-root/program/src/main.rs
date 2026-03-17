//! A simple program that takes a number `n` as input, and writes the `n-1`th and `n`th fibonacci
//! number as an output.

// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_primitives::{address, uint, Address, U256};
use bankai_types::ProofBundle;
use bankai_verify::verify_batch_proof;

const SEPOLIA_IDENTITY_MANAGER: Address = address!("b2EaD588f14e69266d1b87936b75325181377076");
const LATEST_ROOT_SLOT: U256 = uint!(0x12e_U256);

pub fn main() {
    let bundle = sp1_zkvm::io::read::<ProofBundle>();

    let result = verify_batch_proof(bundle).expect("Failed to verify batch proof");
    assert_eq!(
        result.evm.storage_slot.len(),
        1,
        "expected exactly one verified storage-slot request"
    );

    let storage_slot = &result.evm.storage_slot[0];
    assert_eq!(
        storage_slot.address, SEPOLIA_IDENTITY_MANAGER,
        "expected proof for the Sepolia World ID Identity Manager"
    );
    assert_eq!(
        storage_slot.slots.len(),
        1,
        "expected exactly one verified storage slot value"
    );
    assert_eq!(
        storage_slot.slots[0].0, LATEST_ROOT_SLOT,
        "expected proof for the World ID latest-root storage slot"
    );

    let slot_value: [u8; 32] = storage_slot.slots[0].1.to_be_bytes();

    sp1_zkvm::io::commit_slice(slot_value.as_slice());
}
