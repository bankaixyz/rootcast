// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ISP1Verifier} from "./ISP1Verifier.sol";

contract WorldIdRootRegistry {
    error ConflictingRoot(uint64 sourceBlockNumber, bytes32 existingRoot, bytes32 newRoot);
    error InvalidVerifier(address verifierAddress);
    error InvalidProgramVKey(bytes32 programVKey);
    error StaleSourceBlock(uint64 sourceBlockNumber, uint64 latestSourceBlock);

    mapping(uint64 => bytes32) public roots;
    bytes32 public latestRoot;
    uint64 public latestSourceBlock;
    address public immutable verifier;
    bytes32 public immutable programVKey;

    event RootReplicated(uint64 indexed sourceBlockNumber, bytes32 indexed root);
    event VerifierConfigured(address indexed verifierAddress, bytes32 indexed programVKey);

    /// @notice Phase 2 verifies SP1 Groth16 proofs against a pinned verifier
    /// contract and the exact program vkey for this guest.
    /// @dev The public values remain `(uint64 sourceBlockNumber, bytes32 root)`
    /// and are ABI-encoded by the guest/backend path.
    constructor(address verifierAddress, bytes32 sp1ProgramVKey) {
        if (verifierAddress == address(0)) {
            revert InvalidVerifier(verifierAddress);
        }
        if (sp1ProgramVKey == bytes32(0)) {
            revert InvalidProgramVKey(sp1ProgramVKey);
        }

        verifier = verifierAddress;
        programVKey = sp1ProgramVKey;
        emit VerifierConfigured(verifierAddress, sp1ProgramVKey);
    }

    function submitRoot(bytes calldata proofBytes, bytes calldata publicValues) external {
        (uint64 sourceBlockNumber, bytes32 root) = abi.decode(publicValues, (uint64, bytes32));

        if (sourceBlockNumber <= latestSourceBlock) {
            revert StaleSourceBlock(sourceBlockNumber, latestSourceBlock);
        }

        ISP1Verifier(verifier).verifyProof(programVKey, publicValues, proofBytes);

        bytes32 existingRoot = roots[sourceBlockNumber];
        if (existingRoot != bytes32(0) && existingRoot != root) {
            revert ConflictingRoot(sourceBlockNumber, existingRoot, root);
        }

        roots[sourceBlockNumber] = root;
        latestSourceBlock = sourceBlockNumber;
        latestRoot = root;

        emit RootReplicated(sourceBlockNumber, root);
    }
}
