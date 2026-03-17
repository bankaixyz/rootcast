// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// @notice Official SP1 verifier interface from Succinct's sp1-contracts repo.
interface ISP1Verifier {
    function verifyProof(bytes32 programVKey, bytes calldata publicValues, bytes calldata proofBytes) external view;
}
