---
status: in_progress
priority: p1
issue_id: "002"
tags: [bankai, sp1, rust, sqlite, solidity, world-id, sepolia]
dependencies: []
---

# Execute Phase 2 proving slice for World ID root replicator

## Problem Statement

Phase 2 needs to turn the Phase 1 shell into one real proving slice. The
replicator must detect a Sepolia World ID root change, persist the exact source
block and tx hash, wait for Bankai finality for that exact L1 block, request
the storage-slot proof bundle for that block, generate one SP1 proof, and
submit the result to Base Sepolia without duplicating work across retries or
restarts.

## Findings

- The active execution document is
  `docs/plans/2026-03-17-003-feat-world-id-root-replicator-phase-2-proving-slice-plan.md`.
- The current replicator still contains only the Phase 1 startup shell.
- Bankai’s recommended production flow is still
  `Bankai::new(...) -> init_batch(...).execute() -> verify_batch_proof(...)`.
- The Bankai OpenAPI inventory still exposes finalized execution-height routes,
  including `GET /v1/ethereum/execution/height`.
- The Sepolia World ID Identity Manager proxy delegates to implementation
  `0x388516729878cb04463e5aee8bf12279bc004d3f`.
- The verified Sepolia implementation emits
  `TreeChanged(uint256 indexed preRoot, uint8 indexed kind, uint256 indexed postRoot)`,
  which is the root-changing event to watch at the proxy address.

## Proposed Solutions

### Option 1: Build a full async worker framework now

**Approach:** Introduce a queue, richer worker abstractions, and multichain
submission hooks immediately.

**Pros:**
- Could scale to later phases without another refactor
- Separates responsibilities early

**Cons:**
- Over-builds Phase 2
- Increases restart and test complexity before the first slice works

**Effort:** 1-2 days

**Risk:** High

### Option 2: Build one sequential, restart-safe slice

**Approach:** Keep one in-process watcher plus one sequential job runner backed
by SQLite state, and route submission only to Base Sepolia.

**Pros:**
- Matches the phase plan exactly
- Keeps retry boundaries visible
- Minimizes moving parts while proving the hardest dependency chain

**Cons:**
- Some interfaces may need small expansion in later phases
- Leaves multichain fan-out for a later cut

**Effort:** Several focused implementation passes

**Risk:** Medium

## Recommended Action

Execute Option 2. Add a small source-aware watcher, a SQLite-backed job runner,
Bankai finalized-height polling, exact-block proof retrieval, SP1 proof
generation, and a Base Sepolia submission path with contract checks for
duplicate or conflicting submissions.

## Technical Details

**Primary documents:**
- `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
- `docs/plans/2026-03-17-003-feat-world-id-root-replicator-phase-2-proving-slice-plan.md`

**Reference examples:**
- `world-id-root/`
- `base-balance/`

**External confirmations:**
- Bankai SDK live docs from `https://docs.bankai.xyz/llms-sdk.txt`
- Bankai OpenAPI summary from
  `/Users/paul/.codex/skills/bankai-api-spec/scripts/openapi_summary.py`
- Sepolia World ID implementation ABI/source via Etherscan

## Resources

- **Master plan:** `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
- **Phase 2 plan:** `docs/plans/2026-03-17-003-feat-world-id-root-replicator-phase-2-proving-slice-plan.md`
- **Phase 1 plan:** `docs/plans/2026-03-17-002-feat-world-id-root-replicator-phase-1-foundation-plan.md`
- **World ID example:** `world-id-root/`
- **Base balance example:** `base-balance/`

## Acceptance Criteria

- [x] One new Sepolia root detection creates exactly one persisted job
- [x] The job waits for Bankai finality of the exact source block
- [x] The Bankai proof bundle is requested for that exact block and slot
- [ ] The SP1 proof path succeeds for the returned bundle
- [ ] Base Sepolia accepts one valid submission and stores the root
- [x] State transitions are restart-safe and idempotent
- [x] Failure states persist enough detail to resume or debug
- [x] The Phase 2 plan is updated as work completes

## Work Log

### 2026-03-17 - Execution setup

**By:** Codex

**Actions:**
- Reviewed the Phase 2 plan and supporting Phase 1/master plan context
- Refreshed the live Bankai SDK docs and recommended SDK path
- Refreshed the Bankai OpenAPI route inventory
- Confirmed the current branch is `experiment/replicator`
- Confirmed the verified Sepolia World ID root-changing event is
  `TreeChanged(preRoot, kind, postRoot)` on the proxy address
- Created this execution todo

**Learnings:**
- The biggest Phase 2 risk is not fetching a proof bundle; it is keeping exact
  source-block identity and restart-safe job boundaries aligned across watcher,
  Bankai, prover, and submission layers

### 2026-03-17 - Phase 2 slice implemented and verified locally

**By:** Codex

**Actions:**
- Implemented the Sepolia watcher, Bankai finalized-height polling, exact-block
  Bankai proof-bundle retrieval, SP1 proof artifact handling, and Base Sepolia
  submission path
- Hardened the SP1 guest so it asserts the expected Bankai proof shape and
  commits ABI-encoded public values that match the backend and Solidity decode
- Locked the registry contract down to one authorized submitter for this phase
  and expanded Foundry coverage for duplicate, conflicting, and unauthorized
  submissions
- Added restart-safe DB transition helpers and a follow-up migration that
  enforces one replication job per observed root for both fresh and existing
  local databases
- Verified the workspace with `cargo fmt --all --check`, `cargo test`, and
  `forge test`
- Updated the Phase 2 plan with completed local acceptance checks and remaining
  live-validation gaps

**Learnings:**
- The critical idempotency invariant for this phase is not just unique observed
  roots; it is unique replication jobs per observed root, otherwise retries can
  still fork the lifecycle
- The public-values encoding boundary needed to be explicit: serde bytes from
  the guest were not sufficient because the contract and backend expect ABI
  decoding
- The slice is locally coherent now, but real-network validation is still
  needed for Bankai proof generation and Base Sepolia submission

### 2026-03-17 - Catch-up coalescing and prove-rate limiting

**By:** Codex

**Actions:**
- Updated the watcher so each poll window keeps only the newest `TreeChanged`
  event and drops older superseded root changes instead of persisting a backlog
- Added a prove-request cooldown so the runner only starts a fresh proof when
  the last requested proof is at least 50 minutes old
- Added structured Bankai finality logs that show the finalized height, the
  required source block, and the wait-versus-advance decision on every poll
- Added a follow-up SQLite migration for `proof_requested_at` and expanded the
  backend test suite to cover coalescing and cooldown behavior

**Learnings:**
- The right cutoff is at the proving boundary, not at submission time. That
  keeps the persisted state simple while still preventing proof storms during
  bursty World ID updates
- Initial catch-up does not need historical intermediate roots for this phase
  because later World ID roots already commit the full tree state

### 2026-03-17 - Remaining Phase 2 work

**By:** Codex

**Actions:**
- Re-reviewed the active Phase 2 plan and this execution todo after the latest
  implementation passes
- Confirmed the core local slice is implemented and the remaining incomplete
  work is now concentrated in live validation rather than local lifecycle code

**Learnings:**
- The still-open acceptance criteria are the real network boundaries:
  successful SP1 proving against a real Bankai bundle, and one successful Base
  Sepolia on-chain submission storing the root
- The next practical Phase 2 step is to run one funded, verifier-bound, end to
  end proof-and-submit exercise against real credentials and the deployed
  registry

### 2026-03-17 - Base Sepolia registry deployed and verified

**By:** Codex

**Actions:**
- Deployed `WorldIdRootRegistry` to Base Sepolia at
  `0xbF6d105433698385293f5280987e8A5b1617d776`
- Pinned the deployment to the direct SP1 v5 Groth16 verifier
  `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5`
- Derived the current program vkey
  `0x00121643a8e0b1426431683ed5bce193445f3c596ad02d126103658502d6af3f`
  from the compiled guest and used it in the constructor
- Verified the deployed contract successfully on Basescan

**Learnings:**
- The contract deployment milestone is complete now, so the remaining live
  Phase 2 risk is no longer deployment plumbing; it is waiting for Bankai
  finality and then exercising one full proof-and-submit cycle end to end
