# World ID root replicator

This example is the in-progress World ID root replicator. It is derived from
[`bankai-sp1-template`](https://github.com/bankaixyz/bankai-sp1-template),
which remains the authoritative source for zkVM workspace layout, toolchain
pins, lockfile behavior, and SP1 build conventions.

The repository contains:

- `backend/` for runtime config, database startup, orchestration, and the
  read-only API
- `program/` for the SP1 guest program
- `contracts/` for the destination-chain root registry and verifier integration
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
- World ID root source constants
- read-only API endpoints for status, roots, chains, and job detail
- a dark frontend landing page and dashboard for replication state

The remaining work is live end-to-end validation against deployed verifier and
registry contracts across the EVM targets and Starknet Sepolia, plus the Phase
5 deployment and productionization work.

For contract deployment and verification, see
[`contracts/README.md`](contracts/README.md) and the helper script
`contracts/script/deploy_registry.sh`.

To deploy the Starknet Sepolia registry with the values already present in
`.env`, run:

```bash
cd contracts/starknet
./deploy.sh
```

To manually submit a stored proof artifact to one configured destination chain,
run:

```bash
cargo run -p world-id-root-replicator-backend --bin submit_proof -- \
  --chain starknet-sepolia \
  --artifact artifacts/proofs/job-1.bin \
  --wait
```

It uses the same env-driven chain configuration as the backend runner. Pass
`--registry 0x...` to override the configured destination contract for a single
debugging run.

To run the backend in API-only mode for UI work, use:

```bash
cargo run -p world-id-root-replicator-backend -- --api-only
```

This keeps the HTTP API available for the frontend but does not start new
watching, proving, or replication work.
