# Contracts

This directory contains the destination-chain verification and storage contract
for the World ID root replicator.

`WorldIdRootRegistry.sol` verifies SP1 proofs on-chain, decodes the ABI-encoded
public values, and stores roots keyed by the exact L1 source block number. The
contract keeps the original `submitRoot(bytes,bytes)` shape, but it now binds
that call to two immutable constructor values:

- the SP1 verifier contract address
- the SP1 program verification key for `world-id-root-replicator-program`

## Verifier integration

The contract calls `ISP1Verifier.verifyProof(programVKey, publicValues,
proofBytes)` before writing any root state. That means a caller no longer needs
to be a trusted submitter. Any caller can relay a proof, but only a proof that
matches both the configured verifier and the configured program vkey can update
state.

For the current SP1 5.2.x workspace, Succinct's official `sp1-contracts`
deployments publish these current testnet addresses:

- `SP1_VERIFIER_GATEWAY_GROTH16`: `0x397A5f7f3dBd538f23DE225B51f532c34448dA9B`
- `V5_0_0_SP1_VERIFIER_GROTH16`: `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5`
- `V5_0_0_SP1_VERIFIER_PLONK`: `0x0459d576A6223fEeA177Fb3DF53C9c77BF84C459`

Those addresses are currently listed for Sepolia (`11155111`), Base Sepolia
(`84532`), OP Sepolia (`11155420`), and Arbitrum Sepolia (`421614`) in
Succinct's `contracts/deployments/*.json` files. Because this backend submits
Groth16 proofs, the direct v5 Groth16 verifier is the strictest version-pinned
option, while the Groth16 gateway is the recommended auto-routing option from
Succinct's README.

## Program vkey

You can print the current SP1 program vkey from the compiled ELF with this
command:

```bash
cargo run -p world-id-root-replicator-backend --bin print_program_vkey
```

Use that value as the `programVKey` constructor argument when you deploy the
registry.

## Deployment

Use the deploy helper from the `contracts/` directory. It now supports Base
Sepolia, OP Sepolia, and Arbitrum Sepolia through one script.

```bash
./script/deploy_registry.sh --chain base
```

The script loads `../.env`, selects the chain-specific `*_RPC_URL`,
`*_PRIVATE_KEY`, and `*_REGISTRY_ADDRESS` variables for the requested chain,
defaults the verifier to the direct SP1 v5 Groth16 verifier
`0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5`, and reads the program vkey from
`PROGRAM_VKEY`.

If `PROGRAM_VKEY` is missing, the script stops and shows the command you need
to run to generate it:

```bash
cargo run -p world-id-root-replicator-backend --bin print_program_vkey
```

If you want to override either constructor argument, pass them explicitly:

```bash
./script/deploy_registry.sh \
  --chain op \
  --verifier 0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5 \
  --program-vkey 0x...
```

The script prints the deployed contract address and the exact
`*_REGISTRY_ADDRESS=...` line to copy into `../.env`.

## Verification

To deploy and verify in one step, pass `--verify` and ensure
`ETHERSCAN_API_KEY` is set in your environment.

```bash
./script/deploy_registry.sh --chain arb --verify
```

The script submits verification to the correct explorer for the selected chain
using the pinned compiler version, optimizer runs, and ABI-encoded constructor
arguments.

## Current Base Sepolia deployment

The current verified Base Sepolia deployment is:

- registry:
  `0xbF6d105433698385293f5280987e8A5b1617d776`
- verifier:
  `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5`
- program vkey:
  `0x00121643a8e0b1426431683ed5bce193445f3c596ad02d126103658502d6af3f`
