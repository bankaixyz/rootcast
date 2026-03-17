---
status: complete
priority: p1
issue_id: "001"
tags: [bankai, sp1, rust, sqlite, solidity, planning]
dependencies: []
---

# Execute Phase 1 foundation for World ID root replicator

## Problem Statement

Phase 1 needs to establish a stable foundation for the World ID root
replicator. If we begin feature work before locking the workspace layout,
template precedence, SQLite schema, SP1 public values, and Solidity contract
surface, later phases will spend time undoing setup mistakes instead of adding
behavior.

## Findings

- The master plan is now explicitly marked as the project-wide master plan in
  `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`.
- The active execution document is
  `docs/plans/2026-03-17-002-feat-world-id-root-replicator-phase-1-foundation-plan.md`.
- The Bankai-recommended zkVM path is to start from
  `https://github.com/bankaixyz/bankai-sp1-template`.
- A local copy of the template already exists in `/tmp/bankai-sp1-template`.
- The template preserves the expected pinned toolchain and lockfile, but it is
  currently on `bankai-sdk` tag `v0.1.2.2`, while local examples in this repo
  use `v0.1.2.3`.
- We kept the Bankai crates on `v0.1.2.3` by explicit user direction so the new
  example stays aligned with the repo's existing Bankai examples.

## Proposed Solutions

### Option 1: Build directly from local examples

**Approach:** Copy `world-id-root` and mutate it into the new app layout.

**Pros:**
- Reuses code already in this repo
- Minimal initial copying

**Cons:**
- Ignores the recommended zkVM setup path
- Risks dependency and workspace drift

**Effort:** 2-3 hours

**Risk:** High

---

### Option 2: Start from the Bankai SP1 template and adapt it

**Approach:** Scaffold `world-id-root-replicator/` from the template, then use
local examples only as logic references.

**Pros:**
- Matches Bankai guidance
- Reduces setup churn later
- Keeps lockfile and toolchain provenance clear

**Cons:**
- Requires a bit more reshaping up front
- Needs careful handling of template versus local example differences

**Effort:** 3-5 hours

**Risk:** Low

## Recommended Action

Execute Option 2. Scaffold the new example from the Bankai SP1 template, adapt
the host crate into `backend/`, create the planned skeleton, add the first
SQLite migration and core types, freeze the `PublicValues` contract, scaffold
the Solidity registry interface, and verify the World ID root source
assumptions.

## Technical Details

**Primary documents:**
- `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
- `docs/plans/2026-03-17-002-feat-world-id-root-replicator-phase-1-foundation-plan.md`

**Reference examples:**
- `world-id-root/`
- `base-balance/`

**Template source:**
- `/tmp/bankai-sp1-template`

## Resources

- **Master plan:** `docs/plans/2026-03-17-001-feat-world-id-root-replicator-plan.md`
- **Phase 1 plan:** `docs/plans/2026-03-17-002-feat-world-id-root-replicator-phase-1-foundation-plan.md`
- **Brainstorm:** `docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md`
- **Bankai SDK docs:** `https://docs.bankai.xyz/llms-sdk.txt`
- **Template:** `https://github.com/bankaixyz/bankai-sp1-template`

## Acceptance Criteria

- [x] `world-id-root-replicator/` is scaffolded from the Bankai SP1 template
- [x] The Phase 1 workspace shape exists
- [x] The Rust workspace builds after reshaping
- [x] The first SQLite migration exists
- [x] Core job and chain state types exist
- [x] Typed SP1 public values exist
- [x] Solidity registry interface scaffold exists
- [x] World ID root source and slot assumptions are verified and documented
- [x] The Phase 1 plan is updated as work completes

## Work Log

### 2026-03-17 - Initial execution setup

**By:** Codex

**Actions:**
- Confirmed active branch `experiment/replicator`
- Reviewed the master plan and the Phase 1 plan
- Confirmed local template availability in `/tmp/bankai-sp1-template`
- Verified the template is the correct Bankai-recommended zkVM starting point
- Created this execution todo

**Learnings:**
- The template versus local-example dependency mismatch needs to be handled
  deliberately
- The biggest early risk is setup drift, not application logic

### 2026-03-17 - Phase 1 implementation and verification

**By:** Codex

**Actions:**
- Scaffolded `world-id-root-replicator/` from `/tmp/bankai-sp1-template` and
  reshaped the host crate into `backend/`
- Added the Phase 1 Rust workspace, backend startup shell, config loader,
  SQLite migration, and core job state types
- Added the typed SP1 `PublicValues` contract in `program/src/main.rs`
- Added the Solidity `WorldIdRootRegistry` scaffold and a minimal Foundry test
- Verified the World ID Sepolia root source constant by comparing
  `latestRoot()` with raw storage slot `0x12e`
- Documented the explicit Bankai crate tag decision to keep `v0.1.2.3`
  instead of reverting to the template's `v0.1.2.2`
- Updated the Phase 1 plan to reflect completion and the final foundation
  decisions

**Tests run:**
- `cargo check`
- `forge build`
- `cargo run -p world-id-root-replicator-backend`
- `cast call --rpc-url "$EXECUTION_RPC" 0xb2EaD588f14e69266d1b87936b75325181377076 "latestRoot()(uint256)"`
- `cast storage --rpc-url "$EXECUTION_RPC" 0xb2EaD588f14e69266d1b87936b75325181377076 0x12e`

**Learnings:**
- The template remains the right structural baseline, but pin alignment needs
  to be documented when repo-local Bankai examples intentionally lead
- The World ID storage-slot constant is safe to carry into Phase 2 only because
  it is now verified against live Sepolia state and documented next to the code

## Notes

- Treat the template as authoritative for setup and pinning during Phase 1,
  except for the explicit Bankai `v0.1.2.3` override.
- Treat local examples as logic references only.
