# Starknet World ID Root Registry

This package is the Starknet Sepolia destination contract for the World ID root
replicator. It mirrors the Solidity registry's core behavior:

- verifies SP1 BN254 Groth16 proofs through Garaga's shared verifier class hash
- pins a single SP1 program verification key at deployment time
- stores roots by exact source block number
- rejects stale source blocks and conflicting roots

## Layout

- `src/lib.cairo`: Cairo contract
- `tests/test_root_registry.cairo`: basic deployment and constant tests
- `deploy.sh`: Starknet Sepolia declare and deploy helper

## Requirements

Install the Starknet toolchain first:

```bash
asdf plugin add scarb
asdf plugin add starknet-foundry
asdf install
```

This repo is pinned to:

- `scarb 2.16.1`
- `starknet-foundry 0.57.0`

Or install `scarb`, `sncast`, and `snforge` however you manage local tooling,
but keep those versions aligned.

## Environment

The deploy script reads the repo root `.env` file and expects:

```bash
STARKNET_SEPOLIA_RPC_URL=https://rpc.starknet-testnet.lava.build/rpc/v0_10
STARKNET_SEPOLIA_PRIVATE_KEY=0x...
STARKNET_SEPOLIA_ACCOUNT_ADDRESS=0x...
WORLD_ID_ROOT_REPLICATOR_PROGRAM_VKEY=0x...
```

It also accepts the existing fallback names:

- `STARKNET_SEPOLIA_RPC`
- `STARKNET_PRIVATE_KEY`
- `STARKNET_ACCOUNT_ADDRESS`
- `PROGRAM_VKEY`

## Build

```bash
scarb build
```

## Test

```bash
snforge test
```

## Deploy

```bash
./deploy.sh
```

Because the script reads the repo root `.env`, this command is enough if you
already set `PROGRAM_VKEY`, `STARKNET_SEPOLIA_RPC_URL`,
`STARKNET_SEPOLIA_PRIVATE_KEY`, and `STARKNET_SEPOLIA_ACCOUNT_ADDRESS`.

On the first run the script imports the OpenZeppelin account into `sncast`,
declares `WorldIdRootRegistry`, deploys it, and prints the
`STARKNET_SEPOLIA_REGISTRY_ADDRESS=...` line to add to `.env`.
