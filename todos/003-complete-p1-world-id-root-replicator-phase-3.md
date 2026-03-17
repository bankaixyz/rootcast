---
status: complete
priority: p1
issue_id: "003"
tags: [bankai, sp1, rust, sqlite, solidity, multichain, world-id, sepolia]
dependencies: ["002"]
---

# Execute Phase 3 multichain fan-out for World ID root replicator

## Problem Statement

Phase 3 needs to turn the current Base-only proving slice into the full
version-one multichain backend. The replicator must keep one proof pipeline per
observed root, fan that proof out to Base Sepolia, OP Sepolia, and Arbitrum
Sepolia, isolate per-chain failures, remain restart-safe, and harden the
registry contract against duplicate and out-of-order updates.

## Findings

- The active execution document is
  `docs/plans/2026-03-17-004-feat-world-id-root-replicator-phase-3-multichain-fanout-plan.md`.
- The current backend schema already has a normalized `chain_submissions`
  table, but the runtime still assumes one destination chain.
- `backend/src/config.rs`, `backend/src/chains/mod.rs`, `backend/src/db.rs`,
  and `backend/src/jobs/runner.rs` are the core files that still encode the
  Base-only assumption.
- `contracts/test/WorldIdRootRegistry.t.sol` already covers duplicate and
  conflicting same-block submits, but not out-of-order latest-root behavior.

## Proposed Solutions

### Option 1: Add parallel per-chain workers now

**Approach:** Spawn separate per-chain submission loops or Tokio tasks and let
them race independently once a proof is ready.

**Pros:**
- Higher theoretical throughput
- Cleaner per-chain isolation at runtime

**Cons:**
- Adds nonce and concurrency complexity immediately
- Makes restart and test behavior harder to reason about
- Goes against the project’s minimal v1 architecture

**Effort:** High

**Risk:** High

### Option 2: Keep one sequential runner and fan out through SQLite

**Approach:** Keep proofing job-level, reuse one proof artifact, and let the
runner advance many `chain_submissions` rows deterministically.

**Pros:**
- Matches the phase plan and repo style
- Preserves Phase 2 restart and idempotency guarantees
- Minimizes new abstractions

**Cons:**
- Leaves throughput optimization for later
- Requires careful aggregate job-state logic

**Effort:** Medium

**Risk:** Medium

## Recommended Action

Execute Option 2. Convert destination config to a three-chain list, generalize
the DB and runner to fan out across all unfinished chain rows, reuse one shared
EVM submission path, add contract tests for out-of-order updates, and verify
the new multichain behavior with backend tests before touching live networks.

## Technical Details

**Primary documents:**
- `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
- `docs/plans/2026-03-17-004-feat-world-id-root-replicator-phase-3-multichain-fanout-plan.md`

**Main implementation files:**
- `world-id-root-replicator/backend/src/config.rs`
- `world-id-root-replicator/backend/src/jobs/types.rs`
- `world-id-root-replicator/backend/src/chains/mod.rs`
- `world-id-root-replicator/backend/src/db.rs`
- `world-id-root-replicator/backend/src/jobs/runner.rs`
- `world-id-root-replicator/contracts/test/WorldIdRootRegistry.t.sol`

## Resources

- **Master plan:** `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
- **Phase 3 plan:** `docs/plans/2026-03-17-004-feat-world-id-root-replicator-phase-3-multichain-fanout-plan.md`
- **Phase 2 todo:** `todos/002-ready-p1-world-id-root-replicator-phase-2.md`

## Acceptance Criteria

- [x] One detected root creates exactly one job and exactly three
      `chain_submissions` rows
- [x] One proof artifact is reused across Base Sepolia, OP Sepolia, and
      Arbitrum Sepolia
- [x] A failure on one chain does not erase or block success on the others
- [x] Restart after partial confirmations does not re-submit confirmed chains
- [x] Duplicate submissions remain idempotent on every destination chain
- [x] Older source blocks do not regress `latestRoot` or `latestSourceBlock`
- [x] The Phase 3 plan is updated as work completes

## Work Log

### 2026-03-17 - Execution setup

**By:** Codex

**Actions:**
- Reviewed the Phase 3 plan and the supporting brainstorm/master-plan context
- Confirmed the active branch is `experiment/replicator`
- Confirmed the current runtime is still Base-only in config, DB helpers, and
  runner wiring
- Created this execution todo

**Learnings:**
- The biggest Phase 3 risk is aggregate state handling, not proof generation
- The existing schema is already close to sufficient, which means the right
  implementation is a focused hard cutover rather than a schema-heavy redesign

### 2026-03-17 - Phase 3 implemented and verified locally

**By:** Codex

**Actions:**
- Converted backend destination config from a Base-only field to an explicit
  three-chain destination list for Base Sepolia, OP Sepolia, and Arbitrum
  Sepolia
- Replaced the Base-specific submission path with one shared EVM submitter in
  `backend/src/chains/mod.rs`
- Generalized the watcher, DB helpers, and runner so one observed root creates
  three `chain_submissions` rows and one proof artifact fans out across them
- Added aggregate job-settlement rules so mixed confirmed and failed chain
  outcomes resolve without overwriting confirmed progress
- Added backend tests for multichain fan-out, partial failure isolation, and
  restart behavior after partial confirmations
- Added a Foundry test that locks the out-of-order contract invariant:
  older source blocks never regress `latestRoot` or `latestSourceBlock`
- Verified the implementation with `cargo test`,
  `cargo test -p world-id-root-replicator-backend`, and `forge test`
- Updated the Phase 3 plan to `status: completed` and checked off its
  acceptance criteria

**Learnings:**
- The cleanest Phase 3 model is to keep proof generation job-scoped and make
  destination progress row-scoped; that removes the need for separate per-chain
  proof jobs or new workflow tables
- The existing schema was sufficient once the job query stopped assuming a
  single joined submission row
- Restart safety at the fan-out layer depends on confirmed-chain immutability:
  once a destination is confirmed, the runner must treat that row as terminal
