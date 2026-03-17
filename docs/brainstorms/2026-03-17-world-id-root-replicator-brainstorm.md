---
date: 2026-03-17
topic: world-id-root-replicator
---

# World ID root replicator

## What we're building

We want to turn the existing `world-id-root` example into a deployable
application that continuously replicates trusted World ID roots from Ethereum
L1 to one or more EVM destination chains.

The application has four parts that work as one product. A backend service
monitors the World ID Identity Manager contract for root changes, records the
exact L1 block where the new root was submitted, waits until that block is
finalized in Bankai's finalized view, requests a Bankai proof bundle for the
exact L1 storage slot at that block, proves the verified result inside SP1, and
submits the resulting proof to destination-chain contracts. SQLite stores job
state, proof metadata, destination-chain sync status, and the latest
replicated root per chain. A small API exposes this state to a frontend. The
frontend explains the project and shows where each root has been replicated, at
which source block, and with what status.

The first version should optimize for reliability and clarity, not maximum
throughput. We only need Ethereum Sepolia as the source chain, EVM L2s as the
destination environment, and a dark-themed read-only status dashboard.

## Why this approach

We could build this as a loose collection of scripts, a proof service plus a
separate relayer, or a single application. The best first version is a single
Rust application with a few small internal modules and one shared SQLite
database.

This keeps the architecture simple while still matching the real lifecycle of
the product: detect, wait for Bankai finality, prove, submit, and report. It
also fits the Bankai-recommended path. The current SDK guidance still points to
one `Bankai` client, `init_batch(None, HashingFunction::Keccak)`,
`ethereum_storage_slot(...)`, and `bankai_verify::verify_batch_proof(...)`
inside the SP1 guest flow. The existing repo example already follows that
shape, which means we can extend a proven pattern instead of designing a new
one.

We should avoid splitting the backend into multiple services until we have a
clear operational need. A queueing system, separate prover workers, or support
for non-EVM targets can all wait.

## Key decisions

- Use Rust for the backend and orchestration layer. This keeps Bankai, SP1, and
  on-chain submission code in one language and lets us reuse the existing
  example structure.
- Keep the Bankai program as a close sibling of the current example. The guest
  should verify the `ProofBundle`, extract the verified root, and commit the
  root plus the source block number as public outputs.
- Model proof generation as an asynchronous job pipeline in one process:
  `detected -> waiting_finality -> proving_bankai_bundle -> proving_sp1 ->
  submitting -> confirmed -> failed`.
- Define `waiting_finality` precisely as waiting until the L1 block that
  emitted the new root is available under Bankai's finalized view. Ethereum
  finality alone is not enough for the proving flow.
- Use SQLite as the source of truth for observed roots, proof attempts,
  transaction hashes, and per-destination-chain replication state.
- Expose a minimal backend API for `status`, `latest root`, `recent updates`,
  and `destination chains`.
- Build an EVM verifier contract that accepts the SP1 proof, stores roots by
  source block number, and tracks the latest trusted root and latest trusted
  block.
- Treat destination chains as config, not code branches. The backend should run
  the same submission flow for each configured EVM chain.
- Start with three destination chains in version one: Base Sepolia, OP
  Sepolia, and Arbitrum Sepolia.
- Start with a dark-themed read-only frontend that behaves like a polished
  monitoring page, not a wallet app.
- Prove each newly Bankai-finalized root immediately in version one. Do not add
  a debounce window yet.
- Use SP1 network proving as the default proving mode for the deployable
  application.
- Treat the World ID root semantic target as `_latestRoot` exposed by the
  `latestRoot()` getter on the Identity Manager implementation. The current
  example reads storage slot `0x12e` on Sepolia. Because the contract is
  upgradeable, we should re-derive and test that slot during implementation
  instead of trusting the constant blindly.

## Suggested stack

The backend should stay minimal and boring:

- Runtime: Rust with `tokio`
- API: `axum`
- Database: `sqlx` with SQLite
- EVM interaction: `alloy`
- Background work: in-process scheduler and worker loops backed by SQLite state
- Proving: SP1 network proving by default
- Frontend: `Next.js` or `Vite` plus React, depending on how much routing polish
  we want
- Styling: custom dark theme with a small component layer, not a heavy design
  system

My recommendation is `axum` plus SQLite for the backend and `Next.js` for the
frontend. That gives us easy deployment, a nice marketing-style landing page,
and a clean path to a small dashboard without adding separate infra.

## Product shape

The user-facing product should have two surfaces.

The first surface is a short landing page that explains the flow in plain
language: watch the L1 root, wait until the root's L1 submission block is
finalized by Bankai, prove the root with Bankai, wrap it in an SP1 proof, and
replicate it to destination chains. This page should make the trust model
legible.

The second surface is an operations dashboard. It should show the current
trusted root, the source Ethereum block, destination chains, the last successful
submission on each chain, recent replication attempts, and failure states when
something breaks. This is enough for a first release. We do not need wallets,
admin actions, replay controls, or per-user authentication yet.

## Resolved questions

- Version one is Sepolia-only on the source side.
- Version one targets three EVM L2 destinations: Base Sepolia, OP Sepolia, and
  Arbitrum Sepolia.
- The frontend is read-only in version one.
- Version one proves each newly Bankai-finalized root immediately.
- SP1 network proving is the default production proving path.

## References from research

The current repo example already proves the World ID root from the Sepolia
Identity Manager at `0xb2EaD588f14e69266d1b87936b75325181377076` and reads slot
`0x12e`. Bankai's current SDK guidance still recommends the batch-builder path
for this flow. The verified Sepolia implementation source also shows that the
semantic root field is `_latestRoot`, exposed by `latestRoot()`.

## Next steps

Use this brainstorm as the single source for `/ce:plan`. The plan should break
the work into four tracks that still land as one application:

1. Backend service and database.
2. Bankai program plus SP1 proof flow.
3. EVM verifier and root registry contracts.
4. Frontend and deployment wiring.
