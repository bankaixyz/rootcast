---
status: in_progress
priority: p1
issue_id: "004"
tags: [starknet, cairo, rust, sp1, bankai, frontend, multichain, world-id, sepolia]
dependencies: ["003"]
---

# Execute Starknet Sepolia as the first non-EVM replication target

## Problem Statement

The current `world-id-root-replicator` can fan out one SP1 proof artifact to
multiple EVM destination chains, but it still assumes every destination is
EVM-shaped. We need to add Starknet Sepolia as the first non-EVM replication
target without forking the architecture into separate pipelines.

This work must add:

- a Starknet Cairo contract with the same root-registry semantics as the EVM
  registry
- Starknet deployment automation
- backend fan-out to Starknet Sepolia
- frontend/API support so Starknet appears as a first-class destination

## Findings

- The active work document is
  `docs/plans/2026-03-17-007-feat-starknet-sepolia-replication-target-plan.md`.
- The current proof artifact flow in
  `world-id-root-replicator/backend/src/proving/sp1.rs` already persists the
  SP1 proof artifact plus ABI-encoded public values, so the first technical
  question is whether that artifact can feed Garaga directly.
- The current backend still hard-codes EVM assumptions in
  `backend/src/config.rs`, `backend/src/jobs/types.rs`, and
  `backend/src/chains/mod.rs`.
- The current read model is generic enough for another destination, but the
  frontend metadata only knows Base/OP/Arbitrum explorer URLs.
- The user pointed to `/tmp/settlement-demo` as the reference implementation
  for Starknet contract, deployment, and submission flow.

## Proposed Solutions

### Option 1: Build a separate Starknet pipeline

**Approach:** Create a separate worker or code path dedicated to Starknet,
including its own proof export and submission flow.

**Pros:**
- Simplifies chain-family-specific code locally
- Avoids touching some existing EVM assumptions

**Cons:**
- Breaks the original architecture
- Duplicates workflow logic
- Makes mixed-chain status and retries harder to reason about

**Effort:** High

**Risk:** High

### Option 2: Keep one fan-out pipeline and add a Starknet submitter

**Approach:** Preserve the existing one-job/one-proof model, extend destination
config with Starknet, add a Cairo contract plus deployment script, and route
the final submission hop through a Starknet-specific submitter.

**Pros:**
- Matches the project architecture and prior plans
- Minimizes state-model churn
- Keeps frontend/API behavior consistent

**Cons:**
- Requires careful removal of EVM-only assumptions
- Introduces a second deployment/toolchain stack

**Effort:** High

**Risk:** Medium

## Recommended Action

Execute Option 2. Start with the proof-calldata spike and backend seams, then
land the Cairo contract and Starknet deployment tooling, and finally wire
Starknet into the existing API/frontend projection.

## Technical Details

**Primary documents:**
- `docs/plans/2026-03-17-007-feat-starknet-sepolia-replication-target-plan.md`
- `docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md`

**Main implementation files:**
- `world-id-root-replicator/backend/src/config.rs`
- `world-id-root-replicator/backend/src/jobs/types.rs`
- `world-id-root-replicator/backend/src/chains/mod.rs`
- `world-id-root-replicator/backend/src/jobs/runner.rs`
- `world-id-root-replicator/backend/src/proving/sp1.rs`
- `world-id-root-replicator/frontend/lib/chain-metadata.ts`
- `world-id-root-replicator/frontend/lib/api.ts`
- `world-id-root-replicator/contracts/starknet/*`

## Resources

- **Plan:** `docs/plans/2026-03-17-007-feat-starknet-sepolia-replication-target-plan.md`
- **Reference repo:** `/tmp/settlement-demo`
- **Prior multichain work:** `todos/003-complete-p1-world-id-root-replicator-phase-3.md`

## Acceptance Criteria

- [x] Backend supports `starknet-sepolia` as a destination without splitting the
      proof/job pipeline
- [x] SP1 proof artifacts can be transformed into Starknet submission calldata
- [x] Cairo registry contract verifies proofs, stores roots, and rejects stale
      or conflicting updates
- [ ] Starknet Sepolia deployment script works from this repo
- [x] API/frontend show Starknet as a first-class destination
- [ ] Rust and Cairo tests cover the new path
- [x] Docs and the execution plan are updated as work completes

## Work Log

### 2026-03-17 - Execution setup

**By:** Codex

**Actions:**
- Reviewed the Starknet target plan and current replicator state
- Confirmed this worktree is on detached `HEAD` and aligned with the user to
  continue working directly in the worktree
- Identified the highest-risk seam as Starknet proof calldata generation from
  the current SP1 artifact
- Created this execution todo

**Learnings:**
- The project is already close to chain-family neutrality at the DB/API layer
- The biggest unknown is not state management but the exact Starknet proof
  submission shape

### 2026-03-17 - Backend, contract, and frontend integration landed

**By:** Codex

**Actions:**
- Added `starknet-sepolia` as a first-class destination in backend config,
  proof fan-out, API projection, and frontend metadata
- Split submission clients into EVM and Starknet implementations and added
  Garaga-based Starknet calldata generation from the existing SP1 proof
  artifact
- Added a Cairo registry package under `world-id-root-replicator/contracts/starknet`
  with a deploy helper and baseline tests
- Updated `.env.example`, project docs, and contract docs for the new chain
- Validated the Rust backend with `cargo test -p world-id-root-replicator-backend`
  and the frontend with `npm run build`

**Learnings:**
- Starknet's canonical Sepolia chain ID does not fit safely in the old
  `u64`/JavaScript `number` path, so chain IDs now need string treatment in the
  read model
- The current SP1 artifact format is sufficient for Garaga calldata generation;
  no second proof-export pipeline was needed
- The remaining gap is toolchain/runtime validation for Cairo and live Sepolia
  deployment because `scarb`, `snforge`, and `sncast` are not installed in this
  workspace
