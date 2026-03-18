# EVM contracts

This Foundry package contains the EVM destination-chain root registry for the
World ID root replicator.

`WorldIdRootRegistry.sol` verifies SP1 proofs on-chain, decodes the ABI-encoded
public values, and stores roots keyed by the exact L1 source block number. The
contract keeps the original `submitRoot(bytes,bytes)` shape, but it now binds
that call to two immutable constructor values:

- the SP1 verifier contract address
- the SP1 program verification key for `world-id-root-replicator-program`

## Build and test

Build the package with Foundry:

```bash
forge build
```

Run the contract tests:

```bash
forge test
```

## Deploy

Use the top-level contract deploy entrypoint for normal deploys:

```bash
../deploy.sh --chain base-sepolia
```

If you need the package-specific deploy helper directly, build first and then
run:

```bash
./deploy.sh --chain base-sepolia
```

The deploy helper accepts only canonical chain names:

- `base-sepolia`
- `op-sepolia`
- `arbitrum-sepolia`
- `chiado`
- `monad-testnet`
- `hyperevm-testnet`
- `tempo-testnet`
- `megaeth-testnet`
- `plasma-testnet`

Pass `--verify` to verify a deployment after deploy:

```bash
./deploy.sh --chain arbitrum-sepolia --verify
```

Verification is currently skipped with a warning for these chains:

- `monad-testnet`
- `hyperevm-testnet`
- `tempo-testnet`
- `megaeth-testnet`
- `plasma-testnet`

Deploys still go through for those chains when `--verify` is passed.

The script prints the exact `*_REGISTRY_ADDRESS=...` line to paste into
`.env`.
