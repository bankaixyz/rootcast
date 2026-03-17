---
title: feat: Phase 2 proving pipeline vertical slice for World ID root replicator
type: feat
status: active
date: 2026-03-17
origin: docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md
---

# feat: Phase 2 proving pipeline vertical slice for World ID root replicator

## Overview

This plan narrows the master plan to Phase 2 only. The goal is to prove the
hardest cross-layer dependency with one real vertical slice: detect a new World
ID root on Sepolia, record the exact L1 source block, wait until that block is
finalized in Bankai's finalized view, fetch the exact-block storage proof
bundle, generate one SP1 network proof, and submit it to exactly one
destination chain.

This phase carries forward the brainstorm decisions and the completed Phase 1
foundation. Use Base Sepolia as the only destination chain in this phase. Do
not broaden to OP Sepolia, Arbitrum Sepolia, frontend work, or deployment work
yet.

See brainstorm:
`docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md`
See master plan:
`docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
See Phase 1:
`docs/plans/2026-03-17-002-feat-world-id-root-replicator-phase-1-foundation-plan.md`

## Problem statement

Phase 1 froze the workspace, schema shell, public values, and contract surface,
but the product is still not operational. The main risk now is proving the
full source-block-aware lifecycle incorrectly.

This phase must preserve one critical rule from the brainstorm: after detecting
a root update, we wait for the exact L1 block that submitted that root to
become finalized in Bankai's finalized view. Ethereum finality alone is not a
sufficient proving condition.

The second major risk is hidden idempotency failure. Detection, finality
polling, proof retrieval, SP1 network proving, and one-chain submission all
span separate systems. If we do not define retry boundaries and state
transitions carefully now, restarts and transient failures will create
duplicate jobs, duplicate proofs, or conflicting on-chain submissions.

## Proposed solution

Implement one in-process vertical slice with SQLite-backed state and no queue
broker:

1. detect one new root-changing event on Sepolia
2. persist the root, source block number, and source tx hash
3. create one replication job
4. poll Bankai finalized execution height until the exact source block is
   finalized in Bankai's view
5. request one exact-block storage-slot proof bundle with the Bankai SDK
6. verify that proof bundle inside the SP1 guest and request one SP1 network
   proof
7. submit the proof to Base Sepolia
8. persist the final on-chain result

Use the Bankai SDK path first, not raw HTTP, for production logic:
`Bankai::new(...)` -> `api.ethereum().execution().height(finalized)` ->
`init_batch(None, HashingFunction::Keccak)` ->
`ethereum_storage_slot(...)` -> `execute()` ->
`bankai_verify::verify_batch_proof(...)`

Use the Bankai API spec only to confirm route names and response shapes when
debugging or if we need an HTTP fallback later. The local config still points
to `https://sepolia.api.bankai.xyz` and
`https://sepolia.api.bankai.xyz/v1/openapi.json`, but the route inventory
should be refreshed once network access is available again.

## Scope of this phase

Phase 2 includes:
- Sepolia root-change detection
- exact source-block persistence
- Bankai finalized-height polling
- exact-block proof-bundle retrieval
- SP1 guest hardening for the actual proof shape
- SP1 network proving integration
- one-chain Base Sepolia submission
- SQLite-backed retry and lifecycle state
- minimal contract hardening required for safe single-chain submission

Phase 2 excludes:
- OP Sepolia and Arbitrum Sepolia fan-out
- read-only API expansion beyond what the worker needs
- frontend rendering
- deployment automation
- operator controls and replay tools

## Research findings that shape this phase

- The brainstorm fixed the core behavioral requirement: wait for Bankai finality
  of the exact L1 source block, not generic Ethereum finality.
- The master plan defines Phase 2 as a proving pipeline vertical slice with one
  destination chain only.
- The existing `world-id-root` example already shows the right Bankai SDK shape:
  `Bankai::new(...)`, `BankaiBlockFilterDto::finalized()`,
  `init_batch(None, HashingFunction::Keccak)`, `ethereum_storage_slot(...)`,
  and `execute()`.
- The existing `world-id-root` guest shows the minimal
  `verify_batch_proof(...)` path.
- The existing `base-balance` guest shows the preferred typed public-values
  pattern.
- Phase 1 already froze:
  - `backend/src/jobs/types.rs`
  - `program/src/main.rs`
  - `contracts/src/WorldIdRootRegistry.sol`
  - the Sepolia Identity Manager constant and storage slot

## File and module plan

Create or fill these files:

- `world-id-root-replicator/backend/src/world_id/watcher.rs`
  source-chain detection loop
- `world-id-root-replicator/backend/src/world_id/mod.rs`
  export watcher and source constants
- `world-id-root-replicator/backend/src/bankai/finality.rs`
  Bankai finalized-height polling for one source block
- `world-id-root-replicator/backend/src/bankai/proof_bundle.rs`
  exact-block bundle request logic
- `world-id-root-replicator/backend/src/proving/sp1.rs`
  SP1 network prover client and artifact handling
- `world-id-root-replicator/backend/src/chains/base_sepolia.rs`
  one-chain submission path
- `world-id-root-replicator/backend/src/chains/mod.rs`
  thin shared interface that routes only to Base Sepolia in this phase
- `world-id-root-replicator/backend/src/jobs/runner.rs`
  sequential job executor for the vertical slice
- `world-id-root-replicator/backend/src/db.rs`
  small query helpers for phase state transitions
- `world-id-root-replicator/program/src/main.rs`
  harden proof-shape assumptions before commit
- `world-id-root-replicator/contracts/src/WorldIdRootRegistry.sol`
  replace unsafe placeholder acceptance with verifier-backed or explicitly
  locked-down single-phase behavior
- `world-id-root-replicator/contracts/test/WorldIdRootRegistry.t.sol`
  cover duplicate and conflicting submissions

## Phase steps

### Step 1: confirm the source detection contract event
Deliverables:
- identify the exact Sepolia root-changing event and emitted root field
- document the event signature and detection query strategy
- decide whether v1 watcher uses log polling or a simple indexed event filter

Acceptance checks:
- one observed root maps to one exact source block and one source tx hash
- the watcher does not infer source blocks from storage diffs alone

### Step 2: implement source-aware detection and job creation
Deliverables:
- watcher loop persists `observed_roots`
- one new replication job is created idempotently
- duplicate detections reuse the same persisted record

Acceptance checks:
- detecting the same root twice does not create a second job
- job state starts in `WaitingFinality` immediately after persistence

### Step 3: implement Bankai finality polling
Deliverables:
- poll Bankai finalized execution height through the SDK
- compare finalized height against the persisted source block
- mark the observed root finalized only when Bankai finalized height reaches or
  exceeds that source block

Acceptance checks:
- finalized-height polling is source-block-aware
- the job never requests a proof bundle before Bankai finality

### Step 4: implement exact-block proof-bundle retrieval
Deliverables:
- build one Bankai batch for the Identity Manager storage slot at the exact
  source block
- persist enough proof metadata to resume or diagnose failures
- classify retryable versus terminal Bankai errors

Acceptance checks:
- the proof bundle request uses the persisted source block, not the current
  latest finalized block
- the returned proof bundle is tied to the expected root slot request

### Step 5: harden the SP1 guest and network proving path
Deliverables:
- assert the expected Bankai proof shape before indexing storage results
- request one SP1 network proof from the backend
- persist proof artifact reference and job state transitions

Acceptance checks:
- guest public values remain exactly `{source_block_number, root}`
- malformed or unexpected bundle shape fails explicitly
- successful proofs can be verified and decoded by the backend and contract path

### Step 6: implement one safe Base Sepolia submission path
Deliverables:
- wire one Base Sepolia contract client
- replace the unsafe placeholder contract write path with a safe phase-2 write
  path
- persist tx hash and final submission state

Acceptance checks:
- one valid proof updates Base Sepolia registry state
- duplicate resubmission is idempotent
- conflicting resubmission for the same source block reverts cleanly

### Step 7: make the slice restart-safe
Deliverables:
- define retry boundaries for watcher, finality polling, proof retrieval,
  proving, and submission
- ensure one in-progress job can resume after backend restart
- add focused integration coverage around state transitions

Acceptance checks:
- restart during `WaitingFinality` resumes polling without duplicate jobs
- restart after proof generation but before submission does not re-prove
  unnecessarily
- restart after submission but before local confirmation does not double-submit
  blindly

## System-wide impact

- Interaction graph:
  watcher -> SQLite -> finality poller -> Bankai proof request -> SP1 network
  prover -> Base Sepolia submitter -> SQLite
- Error propagation:
  classify retryable RPC/API/prover errors separately from terminal decode,
  verification, or root-mismatch errors
- State lifecycle risks:
  every persisted transition must be idempotent because Phase 2 introduces
  restart and retry behavior for the first time
- API surface parity:
  SQLite state names, Rust enums, guest public values, and Solidity decode
  expectations must remain aligned
- Integration test scenarios:
  root detected before finality, restart during waiting, malformed proof bundle,
  successful proof to Base Sepolia, duplicate submit retry

## Implementation status

Implemented in this phase:
- the Sepolia watcher now polls the World ID `TreeChanged` event and persists
  `root_hex`, `source_block_number`, and `source_tx_hash`
- the backend runner now advances one SQLite-backed job through
  `WaitingFinality -> ReadyToProve -> ProofInProgress -> ProofReady -> Submitting -> Completed`
- Bankai finalized-height polling now gates proof generation on the persisted
  exact source block
- the Bankai proof-bundle request now uses the persisted source block and the
  frozen World ID root storage slot
- the SP1 backend now persists a proof artifact, reloads it after restart, and
  refuses public-values mismatches
- the SP1 guest now commits ABI-encoded `{source_block_number, root}` public
  values so the backend and contract decode the same shape
- Base Sepolia submission is wired for this phase through one on-chain SP1
  verifier path with an immutable program-vkey binding
- SQLite uniqueness now enforces one replication job per observed root with a
  follow-up migration for existing local databases

Still intentionally pending:
- live validation against a real Bankai proof bundle
- live validation of one Base Sepolia submission storing a root on-chain
- final deployment choice between the direct v5 Groth16 verifier and the
  Groth16 gateway
- end-to-end proof generation against real network credentials and contracts

## Acceptance criteria

### Functional requirements
- [x] one new Sepolia root detection creates exactly one persisted job
- [x] the job waits for Bankai finality of the exact source block
- [x] the Bankai proof bundle is requested for that exact block and slot
- [ ] the SP1 network proof succeeds for the returned bundle
- [ ] Base Sepolia accepts one valid submission and stores the root

### Non-functional requirements
- [x] all state transitions are restart-safe and idempotent
- [x] failure states are persisted with enough detail to resume or debug
- [x] the guest and contract still agree on the public-values shape

### Quality gates
- [x] Rust workspace builds after Phase 2 changes
- [x] SQLite migrations apply cleanly on a fresh database
- [x] contract tests cover duplicate and conflicting submissions
- [ ] at least one end-to-end local or testnet validation proves the full slice

## Dependencies and risks

- confirm the exact Sepolia root-changing event ABI before coding the watcher
- confirm the canonical SP1 verifier path we will use on Base Sepolia before
  final contract wiring
- refresh the Bankai OpenAPI route names when network access is available again
- keep Phase 2 single-chain only; do not let multichain fan-out bleed into this
  plan

## Sources and references

### Origin
- `docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md`

### Internal references
- `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
- `docs/plans/2026-03-17-002-feat-world-id-root-replicator-phase-1-foundation-plan.md`
- `world-id-root/script/src/bin/main.rs`
- `world-id-root/program/src/main.rs`
- `base-balance/program/src/main.rs`
- `world-id-root-replicator/program/src/main.rs`
- `world-id-root-replicator/backend/src/jobs/types.rs`
- `world-id-root-replicator/contracts/src/WorldIdRootRegistry.sol`

### Bankai references
- local SDK guidance:
  `/Users/paul/.codex/skills/bankai-sdk/references/sdk-recommended-paths.md`
- local API runtime config:
  `/Users/paul/.codex/skills/bankai-api-spec/config/bankai_api.json`

## Next steps

After this plan is saved, the best follow-up is to execute Phase 2 as one
strict vertical slice. Do not split into backend-only and contract-only tracks
yet.
