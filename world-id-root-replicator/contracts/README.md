# Contracts

This directory contains all destination-chain contracts for the World ID root
replicator.

The tree is split by platform:

- [`evm/`](./evm/README.md) for the Foundry-based EVM contracts and deploy
  helper
- [`starknet/`](./starknet/README.md) for the Cairo contract
- [`solana/`](./solana/README.md) for the Anchor workspace and Solana scripts

## Unified deploy flow

Use the top-level deploy entrypoint for routine deployments:

```bash
./deploy.sh
```

By default, the command rebuilds contracts and then deploys every chain listed
in `ENABLED_DESTINATION_CHAINS` from the repo root `.env` file. It prints one
final block of paste-ready env lines after the full run succeeds.

Deploy one chain by its canonical `.env` name:

```bash
./deploy.sh --chain base-sepolia
```

Skip the default rebuild step when you already have fresh artifacts:

```bash
./deploy.sh --skip-build --chain starknet-sepolia
```

Forward EVM verification with `--verify`:

```bash
./deploy.sh --verify --chain arbitrum-sepolia
```

Verification is currently skipped with a warning for these chains:

- `monad-testnet`
- `hyperevm-testnet`
- `tempo-testnet`
- `megaeth-testnet`
- `plasma-testnet`

Deploys still go through for those chains when `--verify` is passed.

## Supported chain names

The deploy scripts accept only canonical chain names:

- `base-sepolia`
- `op-sepolia`
- `arbitrum-sepolia`
- `chiado`
- `monad-testnet`
- `hyperevm-testnet`
- `tempo-testnet`
- `megaeth-testnet`
- `plasma-testnet`
- `starknet-sepolia`
- `solana-devnet`

## Program vkey

Print the current SP1 program vkey from the compiled ELF with:

```bash
cargo run -p world-id-root-replicator-backend --bin print_program_vkey
```

Use that value as the deployment-time program key when you deploy the registry
contracts.
