---
status: ready
priority: p1
issue_id: "004"
tags: [rust, solana, anchor, backend, frontend, world-id-root-replicator]
dependencies: []
---

# Add Solana Devnet destination support

This todo tracks execution of the Solana Devnet destination-support plan for
the World ID root replicator.

## Problem Statement

The current replicator can prove one World ID root update and fan that proof
out to three EVM destinations, but it cannot deploy to, submit to, confirm on,
or visualize a Solana target. The user wants Solana Devnet to be the first
non-EVM destination in the product.

## Findings

- The current destination enum only includes Base Sepolia, OP Sepolia, and
  Arbitrum Sepolia in
  `world-id-root-replicator/backend/src/jobs/types.rs`.
- Backend submission is currently hard-wired to `EvmSubmitter` in
  `world-id-root-replicator/backend/src/jobs/runner.rs`.
- The SP1 guest already emits ABI-encoded `(source_block_number, root)` public
  values, so the proof artifact itself can stay unchanged.
- The user-provided `settlement-demo` repo shows a clean Solana pattern using
  Anchor, PDAs, `sp1_solana::verify_proof`, and flexible Solana key loading.
- The frontend already renders per-target state, so Solana mostly needs new
  metadata and generic target copy.

## Proposed Solutions

### Option 1: Add a minimal Solana target path in the existing product

**Approach:** Add a small Anchor workspace, a `SolanaSubmitter`, and frontend
metadata while reusing the existing proof artifact and queue model.

**Pros:**
- Preserves the one-proof, one-backend design
- Reuses the strongest local patterns already present
- Ships the requested feature directly

**Cons:**
- Requires Solana toolchain setup
- Touches backend, docs, and frontend together

**Effort:** Medium

**Risk:** Medium

---

### Option 2: Build a separate Solana relay service

**Approach:** Keep the current replicator EVM-only and add a second process just
for Solana deployment and submission.

**Pros:**
- Avoids some backend refactoring

**Cons:**
- Violates the current architecture direction
- Adds avoidable operational complexity
- Splits state and observability

**Effort:** High

**Risk:** High

## Recommended Action

Implement Option 1. Keep the SP1 guest unchanged, add a dedicated Solana
workspace for the registry, extend the backend destination model with a
Solana submitter, and update API/frontend target metadata so Solana appears as
a first-class destination.

## Technical Details

**Affected areas:**
- `world-id-root-replicator/solana/`
- `world-id-root-replicator/backend/src/chains/`
- `world-id-root-replicator/backend/src/config.rs`
- `world-id-root-replicator/backend/src/jobs/`
- `world-id-root-replicator/backend/src/api/`
- `world-id-root-replicator/frontend/lib/`
- `world-id-root-replicator/frontend/components/`

**Database changes:**
- Likely one migration for neutral `target_address` naming if the codebase
  cuts over from EVM-specific `registry_address`.

## Resources

- Plan:
  `docs/plans/2026-03-17-007-feat-solana-devnet-destination-support-plan.md`
- Brainstorm:
  `docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md`
- Reference repo:
  `https://github.com/gxndwana-bankai/settlement-demo`

## Acceptance Criteria

- [x] Solana registry workspace exists and has tests
- [x] Deployment and initialization scripts exist for Solana Devnet
- [x] Backend can submit to and confirm Solana Devnet
- [x] Existing EVM targets still work
- [x] API and frontend show Solana as a target
- [x] Docs reflect Solana setup and usage

## Work Log

### 2026-03-17 - Execution start

**By:** Codex

**Actions:**
- Read the Solana destination-support plan
- Reviewed existing backend, API, and frontend seams
- Reviewed the user-provided `settlement-demo` Solana program and client
- Created this ready todo for execution tracking

**Learnings:**
- The proof artifact shape is already suitable for Solana
- The main work is the new on-chain target and submitter path, not a new proof
  pipeline
- The current worktree is on a detached `HEAD`, and branch creation is blocked
  by the shared git directory's sandbox location

### 2026-03-17 - Implementation and validation

**By:** Codex

**Actions:**
- Added a dedicated Anchor workspace under `world-id-root-replicator/solana/`
- Implemented the Solana registry PDA model, proof verification, and tests
- Added Solana Devnet deployment, preflight, and initialization scripts
- Integrated `SolanaDevnet` into backend config, submission, DB, API, and UI
- Renamed cross-chain target fields from `registry_address` to
  `target_address`
- Validated the backend test suite, Solana program tests, and frontend build
- Attempted a real Devnet deploy with the provided wallet and RPC endpoint

**Learnings:**
- `anchor keys sync` generated and synced the workspace program id
  `CGPJkHwUYwubDNoaLwEMMNqHcHkKz3wB3SKb2ST4i2G1`
- The deploy script needed to use `SOLANA_DEVNET_RPC_URL` directly instead of
  the hard-coded public `devnet` alias

### 2026-03-17 - Devnet deployment

**By:** Codex

**Actions:**
- Deployed the Solana registry program to Devnet
- Initialized the registry PDA with the current SP1 program vkey
- Confirmed the executable program account and initialized state PDA on-chain
- Updated the root README and Solana README with the live deployment details

**Learnings:**
- The deployed program id is
  `CGPJkHwUYwubDNoaLwEMMNqHcHkKz3wB3SKb2ST4i2G1`
- The initialized state PDA is
  `2emanoFQqqozegXYLWb6bjEB1xS1qKZxnPMr8EHKanaJ`
