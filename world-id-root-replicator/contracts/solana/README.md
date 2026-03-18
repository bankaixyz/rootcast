# Solana target

This workspace contains the Solana Devnet root-registry program for the World
ID root replicator.

The Solana target mirrors the EVM registry at a functional level:

- it binds itself to the current SP1 guest verification key during
  initialization
- it accepts the same SP1 proof bytes and ABI-encoded public values
- it stores the latest trusted root and the latest trusted source block
- it stores one PDA-backed record per replicated source block

## Layout

- `programs/world-id-root-registry-solana/` contains the Anchor program
- `tests/root-registry.ts` contains Anchor-side tests
- `script/deploy.sh` deploys the program to Solana Devnet and initializes the
  registry PDA
- `script/initialize_registry.sh` initializes the registry PDA
- `script/preflight.sh` validates local env and prints derived addresses

## Environment

The scripts read these env variables from the repo root `.env` file when
present, or from your current shell when you export them directly:

- `SOLANA_DEVNET_RPC_URL`
- `SOLANA_DEVNET_PRIVATE_KEY`
- `SOLANA_DEVNET_PROGRAM_ID`
- `PROGRAM_VKEY`

For deployment, `SOLANA_DEVNET_PRIVATE_KEY` must be either a JSON-array
keypair or the path to a Solana keypair file. The deploy script converts
JSON-array input into a temporary wallet file for Anchor and Solana CLI
commands.

For backend submission and registry initialization, the Rust helpers also
accept a base58-encoded keypair string.

## Common commands

Build the program:

```bash
anchor build
```

Run the preflight checks:

```bash
./script/preflight.sh
```

Deploy and initialize through the top-level contract entrypoint:

```bash
../deploy.sh --chain solana-devnet
```

If you need the package-specific helper directly, build first and then deploy:

```bash
anchor build
./script/deploy.sh --chain solana-devnet
```

Your deployer wallet needs enough Devnet SOL to cover program rent,
transaction fees, and the initialization transaction.

## Current Devnet deployment

The current Solana Devnet deployment is:

- program id:
  `CGPJkHwUYwubDNoaLwEMMNqHcHkKz3wB3SKb2ST4i2G1`
- state PDA:
  `2emanoFQqqozegXYLWb6bjEB1xS1qKZxnPMr8EHKanaJ`
- SP1 program vkey:
  `0x00121643a8e0b1426431683ed5bce193445f3c596ad02d126103658502d6af3f`
- deploy signature:
  `2wXRocS8xyQFjm7vPfmEsvWtRzQD69hpUtskejyLtaXK1h9mPv2ipLatHMC5Wb9zTbL74W8pbGaoJHqwGXMkk9EN`
- initialize signature:
  `2p7V1nt8BLz6w31ftsbCcuMky2kXMg2e1M9dQm2tuonKoqDqqW3rgEtGsJQjBTaUap2SXcPmfXC7LYEqfuojzPxq`
