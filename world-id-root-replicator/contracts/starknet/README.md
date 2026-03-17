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

Or install `scarb`, `sncast`, and `snforge` however you manage local tooling.

## Environment

The deploy script reads the repo root `.env` file and expects:

```bash
STARKNET_SEPOLIA_RPC_URL=https://starknet-sepolia.public.blastapi.io/rpc/v0_9
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
./deploy.sh --program-vkey 0x...
```

On the first run the script imports the OpenZeppelin account into `sncast`,
declares `WorldIdRootRegistry`, deploys it with the supplied program vkey, and
prints the `STARKNET_SEPOLIA_REGISTRY_ADDRESS=...` line to add to `.env`.
