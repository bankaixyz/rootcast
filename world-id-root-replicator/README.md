# World ID root replicator

This example is the in-progress World ID root replicator. It is derived from
[`bankai-sp1-template`](https://github.com/bankaixyz/bankai-sp1-template),
which remains the authoritative source for zkVM workspace layout, toolchain
pins, lockfile behavior, and SP1 build conventions.

The repository contains:

- `backend/` for runtime config, database startup, and future orchestration
- `program/` for the SP1 guest program
- `contracts/` for the destination-chain root registry and verifier integration
- `frontend/` for the future read-only dashboard shell

## Provenance

This workspace starts from the Bankai SP1 template and then adapts the host-side
crate into `backend/`. If the local examples in this repository and the template
disagree on setup details, the template wins for Phase 1.

One deliberate exception is the Bankai crate tag. The template currently pins
`bankai-sdk`, `bankai-types`, and `bankai-verify` to `v0.1.2.2`, but this
example keeps `v0.1.2.3` to stay aligned with the existing local Bankai
examples in this repository. That override is intentional and should not be
"corrected" by later cleanup work unless we decide to change all related
examples together.

## Current status

The current implementation includes:

- workspace structure
- pinned Rust and SP1 setup
- SQLite-backed watcher and job lifecycle
- Bankai finalized-height polling
- exact-block proof-bundle retrieval
- SP1 public values and proof artifact handling
- contract-side SP1 verifier and program-vkey binding
- World ID root source constants

The remaining work is live end-to-end validation against deployed verifier and
registry contracts, multichain fan-out, and frontend rendering.
