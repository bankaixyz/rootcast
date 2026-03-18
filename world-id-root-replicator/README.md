# World ID root replicator

This example is the in-progress World ID root replicator. It is derived from
[`bankai-sp1-template`](https://github.com/bankaixyz/bankai-sp1-template),
which remains the authoritative source for zkVM workspace layout, toolchain
pins, lockfile behavior, and SP1 build conventions.

The repository contains:

- `backend/` for runtime config, database startup, orchestration, and the
  read-only API
- `program/` for the SP1 guest program
- `contracts/` for the EVM, Starknet, and Solana destination contracts and
  deploy helpers
- `frontend/` for the read-only landing page and replication dashboard

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
- Starknet Sepolia destination contract and relay path
- Solana Devnet registry workspace, deploy scripts, and backend submitter path
- World ID root source constants
- read-only API endpoints for status, roots, chains, and job detail
- a dark frontend landing page and dashboard for mixed EVM, Starknet, and
  Solana replication state

The remaining work is live end-to-end validation against deployed verifier and
registry contracts across the EVM targets, Starknet Sepolia, and Solana
Devnet, plus the Phase 5 deployment and productionization work.

For contract deployment and verification, see
[`contracts/README.md`](contracts/README.md) and the helper script
`contracts/deploy.sh`.

To manually submit a stored proof artifact to one configured destination chain,
run:

```bash
cargo run -p world-id-root-replicator-backend --bin submit_proof -- \
  --chain solana-devnet \
  --artifact artifacts/proofs/job-3.bin \
  --wait
```

It uses the same env-driven chain configuration as the backend runner. Pass
`--registry <address>` to override the configured destination contract or
program for a single debugging run.

## Current Solana Devnet deployment

The current Solana Devnet deployment is:

- program id:
  `CGPJkHwUYwubDNoaLwEMMNqHcHkKz3wB3SKb2ST4i2G1`
- state PDA:
  `2emanoFQqqozegXYLWb6bjEB1xS1qKZxnPMr8EHKanaJ`
- deploy signature:
  `2wXRocS8xyQFjm7vPfmEsvWtRzQD69hpUtskejyLtaXK1h9mPv2ipLatHMC5Wb9zTbL74W8pbGaoJHqwGXMkk9EN`
- initialize signature:
  `2p7V1nt8BLz6w31ftsbCcuMky2kXMg2e1M9dQm2tuonKoqDqqW3rgEtGsJQjBTaUap2SXcPmfXC7LYEqfuojzPxq`

To run the backend in API-only mode for UI work, use:

```bash
cargo run -p world-id-root-replicator-backend -- --api-only
```

This keeps the HTTP API available for the frontend but does not start new
watching, proving, or replication work.
