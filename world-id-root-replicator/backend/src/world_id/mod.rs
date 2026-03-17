use alloy_primitives::{address, uint, Address, U256};

/// Sepolia World ID Identity Manager proxy.
pub const SEPOLIA_IDENTITY_MANAGER: Address = address!("b2EaD588f14e69266d1b87936b75325181377076");

/// `_latestRoot` is exposed by `latestRoot()` on the Sepolia implementation.
///
/// The current example and the verified Sepolia implementation storage layout
/// place `_latestRoot` at storage slot `0x12e`.
///
/// Verified on March 17, 2026 against Sepolia:
/// - `latestRoot()` returned
///   `0x20eafe3b83608e492a9bf4fa23ebaaeaa25aec3cb16a12f8910068b5e56c2634`
/// - raw storage at slot `0x12e` returned the same value
pub const LATEST_ROOT_SLOT: U256 = uint!(0x12e_U256);

pub mod watcher;
