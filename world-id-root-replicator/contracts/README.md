# Contracts

This directory contains the destination-chain verification and storage contracts
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
Succinct's `contracts/deployments/*.json` files. Chiado (`10200`), Monad
Testnet (`10143`), HyperEVM Testnet (`998`), Tempo Testnet (`42431`), MegaETH
Testnet (`6343`), and Plasma Testnet (`9746`) are not currently listed there,
so this repository only uses explicit verifier addresses for those networks.
Because this backend submits Groth16 proofs, the direct v5 Groth16 verifier is
the strictest version-pinned option on the supported testnets above, while the
Groth16 gateway is the recommended auto-routing option from Succinct's README.

## Program vkey

You can print the current SP1 program vkey from the compiled ELF with this
command:

```bash
cargo run -p world-id-root-replicator-backend --bin print_program_vkey
```

Use that value as the `programVKey` constructor argument when you deploy the
registry.

## EVM deployment

Use the deploy helper from the `contracts/` directory. It now supports Base
Sepolia, OP Sepolia, Arbitrum Sepolia, Gnosis Chiado, Monad Testnet, HyperEVM
Testnet, Tempo Testnet, MegaETH Testnet, and Plasma Testnet through one script.

```bash
./script/deploy_registry.sh --chain base
```

The script loads `../.env`, selects the chain-specific `*_RPC_URL`,
`*_PRIVATE_KEY`, and `*_REGISTRY_ADDRESS` variables for the requested chain,
reads the program vkey from `PROGRAM_VKEY`, and defaults the verifier to the
direct SP1 v5 Groth16 verifier `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5` on
Base Sepolia, OP Sepolia, and Arbitrum Sepolia.

For Chiado, the script requires an explicit verifier address because Succinct's
published deployment list does not currently include chain `10200`. Provide it
with `--verifier`, `CHIADO_SP1_VERIFIER_ADDRESS`, or `SP1_VERIFIER_ADDRESS`.

For Monad Testnet, the script defaults to the pinned deployed verifier
`0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`. You can still override it with
`--verifier`, `MONAD_TESTNET_SP1_VERIFIER_ADDRESS`, or `SP1_VERIFIER_ADDRESS`.

HyperEVM, Tempo, MegaETH, and Plasma Testnet use the same pinned verifier by
default and can each override it with their matching `*_SP1_VERIFIER_ADDRESS`
env var or `--verifier`.

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

Chiado example:

```bash
./script/deploy_registry.sh \
  --chain chiado \
  --verifier 0x... \
  --program-vkey 0x...
```

Monad example:

```bash
./script/deploy_registry.sh \
  --chain monad \
  --program-vkey 0x...
```

HyperEVM example:

```bash
./script/deploy_registry.sh \
  --chain hyperevm \
  --program-vkey 0x...
```

The script prints the deployed contract address and the exact
`*_REGISTRY_ADDRESS=...` line to copy into `../.env`.

## Verification

The deploy helper now covers verification for every EVM chain in this
repository. You can verify as part of a fresh deploy with `--verify`, or you
can verify an existing deployment later with `--address`.

Deploy and verify in one step:

```bash
./script/deploy_registry.sh --chain arb --verify
```

Verify an existing deployment without redeploying:

```bash
./script/deploy_registry.sh \
  --chain monad \
  --address 0x3B40dd0cB126e8d521640407c6A3d663D3EAc7c5 \
  --verify
```

The script submits verification with the pinned compiler version, optimizer
runs, and ABI-encoded constructor arguments. It uses the correct backend for
each chain:

- Base Sepolia, OP Sepolia, Arbitrum Sepolia: Etherscan
- Gnosis Chiado: Blockscout
- Monad Testnet: Etherscan verifier against Monadscan's V1 API
- HyperEVM Testnet: Sourcify via Purrsec
- Tempo Testnet: Sourcify via Tempo's verifier
- MegaETH Testnet: MegaETH Etherscan-compatible API
- Plasma Testnet: Etherscan verifier against PlasmaScan's V1 API

Set the explorer API keys that match the chains you want to verify:

- `ETHERSCAN_API_KEY` for Base, OP, Arbitrum, and MegaETH
- `MONADSCAN_API_KEY` for Monad, with `ETHERSCAN_API_KEY` accepted as a fallback
- `PLASMA_TESTNET_API_KEY` for Plasma, with `ETHERSCAN_API_KEY` accepted as a fallback

Chiado, HyperEVM, and Tempo do not need an API key in this helper. For Monad
and Plasma, Foundry's `etherscan` verifier still requires an API key argument
even when you point it at a custom V1 explorer URL.

## Starknet deployment

The Starknet Sepolia equivalent lives in [`starknet/`](./starknet/README.md).
It pins Garaga's shared SP1 verifier class hash and exposes a
`submit_root(Array<felt252>)` entrypoint so the backend can relay the same SP1
proof artifact to Starknet.

Deploy it from the Starknet package directory:

```bash
cd starknet
./deploy.sh
```

The script imports the configured account into `sncast`, declares the Cairo
contract, deploys it, and prints the exact
`STARKNET_SEPOLIA_REGISTRY_ADDRESS=...` line to copy into `../.env`.

## Current Base Sepolia deployment

The current verified Base Sepolia deployment is:

- registry:
  `0xbF6d105433698385293f5280987e8A5b1617d776`
- verifier:
  `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5`
- program vkey:
  `0x00121643a8e0b1426431683ed5bce193445f3c596ad02d126103658502d6af3f`
