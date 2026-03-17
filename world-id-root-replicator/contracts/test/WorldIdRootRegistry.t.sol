// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import {ISP1Verifier} from "../src/ISP1Verifier.sol";
import {WorldIdRootRegistry} from "../src/WorldIdRootRegistry.sol";

contract WorldIdRootRegistryTest {
    bytes32 internal constant PROGRAM_VKEY =
        0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef;

    WorldIdRootRegistry internal registry;
    MockVerifier internal verifier;
    RelayCaller internal relayCaller;

    function setUp() public {
        verifier = new MockVerifier();
        registry = new WorldIdRootRegistry(address(verifier), PROGRAM_VKEY);
        relayCaller = new RelayCaller();
    }

    function test_submitRootStoresLatestRoot() public {
        uint64 sourceBlockNumber = 12_345;
        bytes32 root = bytes32(uint256(1));
        bytes memory proofBytes = hex"1234";
        bytes memory publicValues = abi.encode(sourceBlockNumber, root);

        verifier.expectVerify(PROGRAM_VKEY, publicValues, proofBytes);
        registry.submitRoot(proofBytes, publicValues);

        require(registry.latestSourceBlock() == sourceBlockNumber, "latestSourceBlock mismatch");
        require(registry.latestRoot() == root, "latestRoot mismatch");
        require(registry.roots(sourceBlockNumber) == root, "stored root mismatch");
    }

    function test_submitRootIsIdempotentForSameValues() public {
        uint64 sourceBlockNumber = 12_345;
        bytes32 root = bytes32(uint256(2));
        bytes memory proofBytes = hex"beef";
        bytes memory publicValues = abi.encode(sourceBlockNumber, root);

        verifier.expectVerify(PROGRAM_VKEY, publicValues, proofBytes);
        registry.submitRoot(proofBytes, publicValues);
        registry.submitRoot(proofBytes, publicValues);

        require(registry.latestSourceBlock() == sourceBlockNumber, "latestSourceBlock mismatch");
        require(registry.latestRoot() == root, "latestRoot mismatch");
        require(registry.roots(sourceBlockNumber) == root, "stored root mismatch");
    }

    function test_submitRootAcceptsValidProofFromAnyCaller() public {
        uint64 sourceBlockNumber = 77;
        bytes32 root = bytes32(uint256(3));
        bytes memory proofBytes = hex"aabbcc";
        bytes memory publicValues = abi.encode(sourceBlockNumber, root);

        verifier.expectVerify(PROGRAM_VKEY, publicValues, proofBytes);
        relayCaller.submit(registry, proofBytes, publicValues);

        require(registry.latestSourceBlock() == sourceBlockNumber, "latestSourceBlock mismatch");
        require(registry.latestRoot() == root, "latestRoot mismatch");
        require(registry.roots(sourceBlockNumber) == root, "stored root mismatch");
    }

    function test_submitRootRejectsInvalidProof() public {
        bytes memory proofBytes = hex"deadbeef";
        bytes memory publicValues = abi.encode(uint64(9), bytes32(uint256(4)));

        verifier.rejectVerify(PROGRAM_VKEY, publicValues, proofBytes);

        (bool ok, bytes memory revertData) =
            address(registry).call(abi.encodeCall(registry.submitRoot, (proofBytes, publicValues)));

        require(!ok, "expected invalid proof revert");
        require(_selector(revertData) == MockVerifier.MockProofRejected.selector, "wrong revert selector");
    }

    function test_submitRootRejectsConflictingRootForSameSourceBlock() public {
        bytes memory firstProofBytes = hex"01";
        bytes memory firstPublicValues = abi.encode(uint64(7), bytes32(uint256(1)));
        verifier.expectVerify(PROGRAM_VKEY, firstPublicValues, firstProofBytes);
        registry.submitRoot(firstProofBytes, firstPublicValues);

        bytes memory secondProofBytes = hex"02";
        bytes memory secondPublicValues = abi.encode(uint64(7), bytes32(uint256(2)));
        verifier.expectVerify(PROGRAM_VKEY, secondPublicValues, secondProofBytes);

        (bool ok, bytes memory revertData) =
            address(registry).call(abi.encodeCall(registry.submitRoot, (secondProofBytes, secondPublicValues)));

        require(!ok, "expected conflicting root revert");
        require(
            _selector(revertData) == WorldIdRootRegistry.ConflictingRoot.selector,
            "wrong revert selector"
        );
    }

    function _selector(bytes memory revertData) internal pure returns (bytes4 selector) {
        require(revertData.length >= 4, "missing selector");
        assembly {
            selector := mload(add(revertData, 0x20))
        }
    }
}

contract RelayCaller {
    function submit(WorldIdRootRegistry registry, bytes memory proofBytes, bytes memory publicValues)
        external
    {
        registry.submitRoot(proofBytes, publicValues);
    }
}

contract MockVerifier is ISP1Verifier {
    error MockProofRejected();
    error UnexpectedProgramVKey(bytes32 actual, bytes32 expected);
    error UnexpectedPublicValues();
    error UnexpectedProofBytes();

    bytes32 internal expectedProgramVKey;
    bytes internal expectedPublicValues;
    bytes internal expectedProofBytes;
    bool internal rejectProof;

    function expectVerify(bytes32 programVKey, bytes memory publicValues, bytes memory proofBytes)
        external
    {
        expectedProgramVKey = programVKey;
        expectedPublicValues = publicValues;
        expectedProofBytes = proofBytes;
        rejectProof = false;
    }

    function rejectVerify(bytes32 programVKey, bytes memory publicValues, bytes memory proofBytes)
        external
    {
        expectedProgramVKey = programVKey;
        expectedPublicValues = publicValues;
        expectedProofBytes = proofBytes;
        rejectProof = true;
    }

    function verifyProof(
        bytes32 programVKey,
        bytes calldata publicValues,
        bytes calldata proofBytes
    ) external view {
        if (programVKey != expectedProgramVKey) {
            revert UnexpectedProgramVKey(programVKey, expectedProgramVKey);
        }
        if (keccak256(publicValues) != keccak256(expectedPublicValues)) {
            revert UnexpectedPublicValues();
        }
        if (keccak256(proofBytes) != keccak256(expectedProofBytes)) {
            revert UnexpectedProofBytes();
        }
        if (rejectProof) {
            revert MockProofRejected();
        }
    }
}
