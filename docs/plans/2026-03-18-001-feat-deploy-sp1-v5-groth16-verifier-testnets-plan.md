---
title: feat: Deploy SP1 v5 Groth16 verifier to requested EVM testnets
type: feat
status: completed
date: 2026-03-18
origin: docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md
---

# feat: Deploy SP1 v5 Groth16 verifier to requested EVM testnets

## Overview

This plan covers one operational outcome: deploy the direct SP1 v5 Groth16
verifier contract from Succinct's `sp1-contracts` repository to the requested
EVM-compatible testnets and produce a deployment report with addresses,
explorer links, and error notes.

The plan intentionally keeps scope narrow. The target deliverable is the direct
`SP1VerifierGroth16` contract used by the local World ID example's verifier
integration path, not a broader verifier-management system. Unless new
requirements emerge during execution, we will not deploy the SP1 gateway.

## Problem statement

The current local example for `world-id-root-replicator` expects an SP1
verifier address that downstream contracts can call directly through
`ISP1Verifier.verifyProof(...)`. That is compatible with the user's stated goal:
be able to verify proofs in the same shape as the World ID replicate example.

However, Succinct's published deployment inventory does not currently cover the
requested testnets as ready-made SP1 v5 Groth16 verifier destinations. The
target networks are therefore not a lookup task. They are first-party
deployment work that needs a repeatable process, per-chain RPC validation, and
a final report.

## Scope

This plan includes:

- deploying the direct `src/v5.0.0/SP1VerifierGroth16.sol:SP1Verifier`
  contract
- targeting these testnets: Monad, HyperEVM, Tempo, MegaETH, Plasma, and
  Gnosis Chiado
- using the provided deployer private key only through environment variables
- validating live RPC chain IDs before each deployment
- recording deployed addresses and explorer links in a Markdown report
- recording chain-specific failures in the same report without blocking the
  other chains

This plan excludes:

- deploying `SP1VerifierGateway` unless the user later asks for gateway routing
- deploying any World ID registry or application-specific wrapper contract
- adding backward-compatibility layers or a generalized deployment framework
- production hardening beyond what is needed to finish these testnet deploys

## Research findings

### Brainstorm decisions carried forward

The relevant World ID brainstorm already established the important product-side
constraint: the intended consumer path is direct proof verification from an SP1
verifier address inside an application contract, not gateway-centric routing
(see brainstorm: `docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md`).

That matches the user's request to "just want to be able to verify proofs" in
the same shape as the World ID replicate example.

### Local code findings

- `world-id-root-replicator/contracts/src/ISP1Verifier.sol` defines the direct
  `verifyProof(bytes32,bytes,bytes)` interface expected by the local example.
- `world-id-root-replicator/contracts/src/WorldIdRootRegistry.sol` stores a
  single immutable verifier address and calls that verifier directly before it
  writes root state.
- `world-id-root-replicator/contracts/README.md` explicitly recommends the
  strict version-pinned direct v5 Groth16 verifier for the current proving
  flow, while describing the gateway as the more general routing option.

### External repo findings

- Succinct's upstream deployment inventory currently includes official
  `V5_0_0_SP1_VERIFIER_GROTH16` deployments for some chains, but not for the
  requested testnet set. The published `contracts/deployments/*.json` files
  include mainnet entries for Monad (`143`), HyperEVM (`999`), MegaETH
  (`4326`), and Plasma (`9745`), plus Sepolia-family testnets, but not the
  requested testnets here.
- The upstream deploy script for v5 deploys the direct verifier and then adds
  it to the gateway route table. That confirms the gateway is an optional
  routing layer on top of the direct verifier, not a prerequisite for proof
  verification.
- The verifier contract itself has no constructor arguments, which makes raw
  `forge create` deployment the simplest correct path for these unsupported
  testnets.
- Upstream `contracts/foundry.toml` already contains RPC aliases for some
  related mainnets, but it does not contain the requested testnet aliases or a
  Gnosis Chiado alias. A custom per-chain deploy flow will therefore be simpler
  than trying to bend the upstream multichain script to new networks mid-task.

### Network findings

The requested networks map to these testnet targets and live chain IDs:

- Monad Testnet: chain ID `10143`, live RPC `https://testnet-rpc.monad.xyz`
- HyperEVM Testnet: chain ID `998`, live RPC
  `https://rpc.hyperliquid-testnet.xyz/evm`
- Tempo Testnet (Moderato): chain ID `42431`, live RPC
  `https://rpc.moderato.tempo.xyz`
- MegaETH Testnet: chain ID `6343`, live RPC `https://carrot.megaeth.com/rpc`
- Plasma Testnet: chain ID `9746`, live RPC `https://testnet-rpc.plasma.to`
- Gnosis Chiado: chain ID `10200`, live RPC `https://rpc.chiadochain.net`

Live RPC validation on March 18, 2026 confirmed those chain IDs through
`eth_chainId`.

### Notable inconsistencies and risks discovered in research

- QuickNode documentation appears to cover some of these ecosystems, but the
  user-provided multi-chain QuickNode URL pattern is not sufficient local proof
  that all six requested testnets are available through that single endpoint.
  Execution should therefore try the user's QuickNode path only when the slug
  is unambiguous and immediately fall back to chain-native public RPCs when it
  is not.
- Tempo has a documentation inconsistency across sources. A QuickNode Tempo doc
  page surfaced chain ID `42429`, while the official Tempo repository README
  and the live RPC both resolve to `42431`. The deploy flow must trust the live
  `eth_chainId` result over secondary docs.
- Explorer support is uneven across these chains. Address pages should be easy
  to link after deployment, but source-code verification automation may differ
  by explorer and is not a prerequisite for the requested deliverable.

## Research decision

External research was necessary for this planning pass.

This task touches live third-party infrastructure, current chain metadata,
current explorer URLs, and current upstream `sp1-contracts` support. That is
exactly the kind of operational work where stale assumptions would create bad
deployment plans.

## Spec flow analysis

The deployment flow has one critical ambiguity and one critical branch.

The ambiguity is "gnosis." Because the user asked for testnets, this plan
interprets that request as Gnosis Chiado, not Gnosis mainnet. Execution should
state that assumption explicitly in the final report.

The critical branch is whether a gateway is required. Current research says no
for the user's stated proof-verification goal:

1. the local example consumes a direct verifier address
2. the direct verifier contract already implements `ISP1Verifier`
3. the gateway only adds route indirection and ownership-managed selector
   routing

That means the smallest correct solution is:

1. deploy the direct verifier everywhere
2. verify that each deployment responds as expected on-chain
3. record the addresses and explorer links
4. stop there unless the user later asks for gateway deployment too

## Proposed solution

Use a hard-cutover deployment workflow centered on the direct verifier only.

The working method is:

1. clone the upstream `sp1-contracts` repository into a temporary working
   directory such as `/tmp/sp1-contracts`
2. build the contracts from `contracts/`
3. deploy `src/v5.0.0/SP1VerifierGroth16.sol:SP1Verifier` to each requested
   testnet with `forge create`
4. confirm the deployed code exists and the chain ID matches the target
5. write one report file with the successful addresses, explorer links, and any
   per-chain failures

This approach avoids unnecessary abstractions:

- no custom Solidity wrapper
- no gateway unless requested
- no upstream deploy-script refactor
- no backward-compatibility support

## Technical approach

### Deployment method

The default deployment command should be the direct Foundry create path from the
upstream repo's `contracts/` directory:

```bash
forge create src/v5.0.0/SP1VerifierGroth16.sol:SP1Verifier \
  --rpc-url "$TARGET_RPC_URL" \
  --private-key "$PRIVATE_KEY"
```

This is simpler than the upstream gateway-aware script because the v5 Groth16
verifier has no constructor arguments and the user's goal does not require
route registration.

### Preflight checks

Before each deployment, the execution flow should:

1. resolve the target RPC URL
2. call `eth_chainId` and compare it with the expected chain ID
3. derive the deployer address from the provided private key
4. check native token balance on the target chain
5. only then broadcast the deployment

If one chain fails preflight, execution should record the failure and continue
to the next chain.

### Output report

Execution should create one report file, for example:

`docs/reports/2026-03-18-sp1-v5-groth16-testnet-deployments.md`

Each chain entry should include:

- chain name
- chain ID
- RPC source used
- deployed verifier address, if successful
- block explorer base URL
- contract address link on the explorer
- transaction hash, if available
- source-verification result, if attempted
- error details, if deployment or verification failed

### Explorer link targets

The report should prefer these explorer families, subject to live validation:

- Monad Testnet:
  `https://testnet.monadvision.com/address/<address>`
- HyperEVM Testnet:
  capture the live explorer URL that resolves after deployment; do not hardcode
  an unverified template ahead of time
- Tempo Testnet:
  `https://explore.tempo.xyz/address/<address>`
- MegaETH Testnet:
  `https://www.megaexplorer.xyz/address/<address>`
- Plasma Testnet:
  `https://testnet.plasmascan.to/address/<address>`
- Gnosis Chiado:
  `https://gnosis-chiado.blockscout.com/address/<address>`

## Alternative approaches considered

### Deploy gateway plus verifier

Rejected for now. This adds ownership, route-registration, and freeze-path
complexity that the user did not ask for. The local World ID example does not
need it.

### Patch the upstream multichain scripts for all target testnets first

Rejected for now. That would require extending upstream-style chain inventory,
potentially custom verifier settings, and additional script plumbing before the
first useful deployment. Raw `forge create` is smaller and safer for this task.

### Reuse existing official deployments

Rejected because the requested networks are testnets and the published
deployments are not available for this target set in a way that satisfies the
task.

## System-wide impact

### Contract-consumer impact

Any downstream contract that currently expects an `ISP1Verifier` address can use
these deployed verifier addresses directly. That is the relevant parity target
for the local World ID example.

### Failure propagation

Failures are isolated per chain. A failed deployment on one testnet should not
block deployment attempts on the other five. The report is the source of truth
for final status.

### State lifecycle risks

The main state risk is not on-chain correctness. It is operator bookkeeping.
Without one report file, the team could lose track of which chain has a live
verifier, which explorer URL is correct, and which failures still need manual
follow-up.

### Integration test scenarios

- deploy to one chain and verify the contract address has non-empty bytecode
- deploy to multiple chains and ensure the recorded chain IDs match the intended
  targets
- feed one deployed verifier address into the World ID example's expected
  verifier slot and confirm the integration shape stays direct
- fail one chain intentionally by using a bad RPC and confirm the other chains
  still execute and the report records the failure

## Implementation phases

### Phase 1: lock scope and pin sources

- [x] Clone `https://github.com/succinctlabs/sp1-contracts` into a temporary
      worktree such as `/tmp/sp1-contracts`
- [x] Record the exact commit SHA used for deployment in
      `docs/reports/2026-03-18-sp1-v5-groth16-testnet-deployments.md`
- [x] Confirm the contract target remains
      `contracts/src/v5.0.0/SP1VerifierGroth16.sol:SP1Verifier`
- [x] Confirm that no gateway deployment is needed for the requested use case

### Phase 2: prepare the per-chain deployment matrix

- [x] Create a deployment matrix in
      `docs/reports/2026-03-18-sp1-v5-groth16-testnet-deployments.md`
- [x] Lock these target rows:
      `monad-testnet`, `hyperevm-testnet`, `tempo-testnet`,
      `megaeth-testnet`, `plasma-testnet`, `gnosis-chiado`
- [x] Validate each target RPC with `eth_chainId`
- [x] Resolve the explorer base URL for each chain
- [x] Decide per chain whether the user's QuickNode URL or the chain-native RPC
      is the safer execution path

### Phase 3: deploy the verifier

- [x] Export `PRIVATE_KEY` in the shell only for the deployment session
- [x] For each chain, run the direct deployment transaction against the verifier
      contract
- [x] Capture the deployed address and transaction hash
- [x] Confirm bytecode exists at the deployed address with `eth_getCode`
- [x] Continue to the next chain even if one chain fails

### Phase 4: verify what can be verified

- [x] Attempt explorer verification only where the explorer and API path are
      clear enough to do safely
- [x] If source verification is not straightforward, still record the address
      link and explicitly mark source verification as skipped or failed
- [x] Prefer correctness of the deployment report over spending excessive time
      on explorer API edge cases

### Phase 5: deliver the report

- [x] Finalize
      `docs/reports/2026-03-18-sp1-v5-groth16-testnet-deployments.md`
- [x] Include one section per chain with success or failure details
- [x] Add a short summary section with total successes and total failures
- [x] Add a short "Assumptions" section stating that "gnosis" was interpreted as
      Gnosis Chiado because the request was for testnets

## Acceptance criteria

### Functional requirements

- [x] The direct SP1 v5 Groth16 verifier contract is deployed to every target
      chain that accepts deployment
- [x] Each successful deployment has a recorded contract address
- [x] Each successful deployment has a recorded explorer address link
- [x] Each failed deployment has a recorded error note
- [x] The final report exists as a Markdown file in `docs/reports/`

### Non-functional requirements

- [x] The private key is never written into repository files
- [x] The deploy flow fails fast on chain ID mismatch
- [x] One chain failure does not stop work on the remaining chains
- [x] The chosen solution stays minimal and does not introduce gateway
      complexity without a user request

### Quality gates

- [x] All recorded chain IDs match live RPC responses
- [x] Every explorer link in the report resolves to the intended contract page
      or is clearly marked unavailable
- [x] The report includes the upstream repo commit SHA and deployment date

## Success metrics

The best outcome is six successful deployments with six working explorer links.

The minimum acceptable outcome for this task is:

- a successful deployment attempt on every requested chain
- a complete report for all six chains
- no silent failures or undocumented partial results

## Dependencies and risks

### Dependencies

- a working local Foundry installation
- access to the upstream `sp1-contracts` repository
- enough testnet funds on the provided deployer key
- working RPC access for each target testnet

### Risks

- the user's QuickNode endpoint may not support every requested testnet
- testnet explorers may not all support the same verification workflow
- one or more chains may have temporary RPC instability or mempool quirks
- HyperEVM testnet explorer URL templates may need live confirmation after the
  deploy instead of ahead of time

## Sources and references

### Origin

- Brainstorm document:
  `docs/brainstorms/2026-03-17-world-id-root-replicator-brainstorm.md`
  Key decisions carried forward: use the direct verifier path for proof
  checking, keep the solution minimal, and optimize for the World ID-style
  proof-verification consumer flow.

### Internal references

- `world-id-root-replicator/contracts/src/ISP1Verifier.sol`
- `world-id-root-replicator/contracts/src/WorldIdRootRegistry.sol`
- `world-id-root-replicator/contracts/README.md`

### External references

- Succinct `sp1-contracts` README:
  [contracts/README.md](https://raw.githubusercontent.com/succinctlabs/sp1-contracts/main/contracts/README.md)
- Succinct v5 deploy script:
  [SP1VerifierGroth16.s.sol](https://raw.githubusercontent.com/succinctlabs/sp1-contracts/main/contracts/script/deploy/v5.0.0/SP1VerifierGroth16.s.sol)
- Succinct v5 verifier contract:
  [SP1VerifierGroth16.sol](https://raw.githubusercontent.com/succinctlabs/sp1-contracts/main/contracts/src/v5.0.0/SP1VerifierGroth16.sol)
- Succinct Foundry config:
  [foundry.toml](https://raw.githubusercontent.com/succinctlabs/sp1-contracts/main/contracts/foundry.toml)
- Monad testnet network info:
  [Monad docs](https://docs.monad.xyz/developer-essentials/testnets)
- HyperEVM testnet network info:
  [Hyperliquid docs](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/hyperevm)
- Tempo testnet network info:
  [Tempo README](https://raw.githubusercontent.com/tempoxyz/tempo/main/README.md)
- MegaETH testnet network info:
  [Alchemy chain page](https://www.alchemy.com/rpc/megaeth-testnet)
- Plasma network information:
  [Plasma docs](https://plasma.to/docs/plasma-chain/network-information/connect-to-plasma)
- Plasma RPC provider notes:
  [Plasma RPC providers](https://www.plasma.to/docs/plasma-chain/tools/rpc-providers)
- Gnosis Chiado network info:
  [Gnosis docs](https://docs.gnosischain.com/about/networks/chiado)

## Final review checklist

- [x] The plan keeps scope on the direct verifier contract only
- [x] The plan reflects the World ID example's direct verifier integration path
- [x] The requested chain list is preserved
- [x] The "gnosis means Chiado" assumption is explicit
- [x] The final Markdown deployment report is named and described
