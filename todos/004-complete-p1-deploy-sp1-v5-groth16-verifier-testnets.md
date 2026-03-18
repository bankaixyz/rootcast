---
status: complete
priority: p1
issue_id: "004"
tags: [sp1, solidity, foundry, deployment, evm, testnet]
dependencies: []
---

# Deploy SP1 v5 Groth16 verifier to requested testnets

## Problem Statement

The requested EVM testnets do not already have a ready-to-use published SP1 v5
Groth16 verifier deployment that satisfies this task. We need to deploy the
direct verifier contract to each requested testnet, confirm the resulting
addresses on-chain, and produce one Markdown report with addresses, explorer
links, and error details.

## Findings

- The active execution document is
  `docs/plans/2026-03-18-001-feat-deploy-sp1-v5-groth16-verifier-testnets-plan.md`.
- The local World ID example consumes a direct `ISP1Verifier` address rather
  than requiring the gateway.
- The upstream `sp1-contracts` v5 deploy script adds a gateway route after
  deploying the direct verifier, which confirms the gateway is optional for the
  user's stated proof-verification goal.
- Live RPC chain ID checks confirm the requested testnet targets are:
  Monad `10143`, HyperEVM `998`, Tempo `42431`, MegaETH `6343`, Plasma
  `9746`, and Gnosis Chiado `10200`.

## Proposed Solutions

### Option 1: Use the upstream multichain deploy scripts

**Approach:** Extend the upstream deploy configuration to support the requested
testnets and route registration.

**Pros:**
- Reuses upstream deployment conventions
- Leaves a path open for gateway routing later

**Cons:**
- More moving parts than the task requires
- Requires extra chain configuration before the first useful deploy

**Effort:** Medium

**Risk:** Medium

### Option 2: Deploy the direct verifier with raw `forge create`

**Approach:** Clone `sp1-contracts`, compile the contracts, and deploy
`src/v5.0.0/SP1VerifierGroth16.sol:SP1Verifier` directly to each target testnet.

**Pros:**
- Smallest correct solution
- Matches the World ID example's verifier-consumer shape
- Avoids unnecessary gateway setup

**Cons:**
- Less reusable than a generalized script
- Explorer verification may need per-chain handling

**Effort:** Low to Medium

**Risk:** Low

## Recommended Action

Execute Option 2. Use the direct v5 Groth16 verifier deployment path, validate
each chain with preflight checks, continue across failures, and record every
outcome in a single Markdown report.

## Technical Details

**Primary documents:**
- `docs/plans/2026-03-18-001-feat-deploy-sp1-v5-groth16-verifier-testnets-plan.md`

**Reference implementations:**
- `world-id-root-replicator/contracts/src/ISP1Verifier.sol`
- `world-id-root-replicator/contracts/README.md`

**External sources:**
- `https://github.com/succinctlabs/sp1-contracts`
- chain docs and explorers referenced in the plan

## Resources

- **Execution plan:**
  `docs/plans/2026-03-18-001-feat-deploy-sp1-v5-groth16-verifier-testnets-plan.md`
- **Report target:**
  `docs/reports/2026-03-18-sp1-v5-groth16-testnet-deployments.md`

## Acceptance Criteria

- [x] The verifier source is cloned and compiled successfully
- [x] Each requested chain passes or fails a documented preflight check
- [x] A deployment attempt is made on every requested testnet
- [x] Each successful deployment has a contract address and explorer link
- [x] Each failure is documented with the reason
- [x] The execution plan is updated as work completes

## Work Log

### 2026-03-18 - Execution setup

**By:** Codex

**Actions:**
- Read the deployment plan and confirmed the scope stays on the direct SP1 v5
  Groth16 verifier
- Confirmed the current branch is `codex/add-gnosis-chiado-support`
- Confirmed local Foundry tooling is available
- Created this execution todo

**Learnings:**
- The key scope decision is already resolved: the gateway is not needed for the
  user's proof-verification goal
- The main operational risk is uneven RPC and explorer support across the
  requested testnets, not the verifier contract itself

### 2026-03-18 - Deployments completed

**By:** Codex

**Actions:**
- Cloned `succinctlabs/sp1-contracts` at commit
  `22c4a47cd0a388cb4e25b4f2513954e4275c74ca`
- Compiled the direct verifier contract with `solcjs 0.8.20` after `forge
  build` failed in the sandbox with `Operation not permitted (os error 1)`
- Ran preflight checks against Monad Testnet, HyperEVM Testnet, Tempo Testnet,
  MegaETH Testnet, Plasma Testnet, and Gnosis Chiado
- Deployed the direct SP1 v5 Groth16 verifier successfully to all six target
  chains
- Confirmed non-empty on-chain bytecode at
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` on all six chains
- Wrote
  `docs/reports/2026-03-18-sp1-v5-groth16-testnet-deployments.md`
- Updated the execution plan to completed

**Learnings:**
- Because the deployer key had the same nonce on each target chain, the direct
  verifier landed at the same contract address on all six networks
- Chiado required a fallback deployment with legacy transaction settings and a
  manual gas price even though the default send path failed
