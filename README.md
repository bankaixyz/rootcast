# World ID Rootcast

World ID should be on every chain. This project makes that practical by
broadcasting the canonical World ID Merkle root from Ethereum L1 to
destination chains across the ecosystem -- so applications can verify
proof of personhood locally, wherever they run, against the same
canonical human set.

Rootcast is powered by [Bankai](https://docs.bankai.xyz), a
stateless light client that proves Ethereum state with zero-knowledge
proofs. Instead of bridging messages or running a light-client contract
on every destination, the system proves the exact L1 storage value and
submits the verified root to each chain's identity registry. One proof
pipeline, many destinations, no per-chain infrastructure to maintain.

## Why broadcast the root?

Under the hood, World ID proof of personhood is a Merkle tree. Users
prove membership against its current root, and applications check that
proof to confirm a unique human is behind the action. Today, that root
lives on Ethereum.

If an application runs on a different chain, it has to reach back to
Ethereum to trust that root. That means either depending on a bridge to
relay the value, or asking users to verify against a remote chain -- both
of which add trust assumptions, latency, or integration friction that
shouldn't be necessary.

Broadcasting the root removes that bottleneck. Each destination chain
holds the same canonical value locally, so verification stays fast,
cheap, and self-contained.

## Why Bankai

Bankai is built around **stateless light clients**. A conventional light
client requires deploying a verifier contract on every destination chain,
initializing it with trusted source-chain state, and continuously syncing
it with on-chain transactions every time validator sets rotate or new
checkpoints finalize. That sync has to happen on every chain, in
parallel, perpetually -- even when nobody is using the client.

Bankai removes that destination-side burden entirely. It syncs the
Ethereum light client fully off-chain, compresses the trusted state into
recursive zero-knowledge proofs, and lets any verifier confirm the result
on demand. The verification interface doesn't need persistent state: you
check one proof, and if it verifies, the data is valid according to
Ethereum's consensus.

For this project, the key properties are:

- **No destination-side light-client contracts.** There is nothing to
  deploy or keep in sync on each target chain.
- **No ongoing sync transactions.** The proof carries the full chain of
  trust forward. Destinations don't need to track validator rotations or
  committee handoffs.
- **Portable verification.** The same proof works on any chain with a
  Groth16 verifier -- EVM, non-EVM, or inside another ZK program.
- **Source-chain finality.** Bankai follows Ethereum's native finality
  model exactly. Roots are broadcast from finalized L1 state, not from
  optimistic or unconfirmed updates.

### What Bankai does not remove

Bankai removes the destination-side infrastructure problem, but the rest
of the system is still stateful. The Ethereum source chain is obviously
stateful. Bankai's off-chain prover tracks source-chain data and
maintains the proof pipeline. And like all proof-of-stake light clients,
the system starts from a trusted bootstrap checkpoint due to
[weak subjectivity](https://ethereum.org/en/developers/docs/consensus-mechanisms/pos/weak-subjectivity/)
-- after which trust is carried forward cryptographically through
recursive proofs.

## Why not a bridge or traditional light client?

This system could be built on a traditional messaging bridge or a
stateful on-chain light client. Both work, but they come with costs that
make universal broadcasting harder.

| | Bridge | Stateful light client | This approach (Bankai) |
|---|---|---|---|
| **Trust surface** | Relayer, validator set, or oracle network | Source-chain consensus + bootstrap | Source-chain consensus + bootstrap |
| **Route availability** | Only works where a bridge route exists and is maintained | Only works where a contract is deployed and synced | Any chain with a Groth16 verifier |
| **Per-chain cost** | Bridge fees + route maintenance | Contract deployment + perpetual sync transactions | Registry contract only |
| **Adding a new chain** | Wait for the bridge to support it | Deploy and initialize a new light-client contract | Deploy a registry, start submitting proofs |

The practical difference shows up most clearly at the margin. If you want
World ID on a new appchain, an emerging L2, or inside a client-side ZK
program, a bridge requires someone to build and maintain a route to that
environment. A stateful light client requires deploying and perpetually
syncing a contract there. With Bankai, the only destination-side
requirement is a Groth16 verifier and a registry contract to store the
verified roots.

## How it works

The broadcast pipeline has four stages:

1. **Observe** -- monitor the World ID identity manager contract on
   Ethereum for new Merkle root updates.
2. **Finalize** -- wait for the source block to reach consensus finality
   on Ethereum L1. Roots are never broadcast from unfinalized state.
3. **Prove** -- generate a zero-knowledge storage proof using Bankai's
   stateless light client. The proof attests to the exact storage value at
   a finalized L1 block, verified inside SP1.
4. **Broadcast** -- submit the proven root to identity registry contracts
   on every destination chain. Each registry stores the verified value
   locally, making it available for on-chain proof-of-personhood checks.

## Current status

This is a proof-of-concept running on testnets. The system demonstrates
end-to-end broadcasting from Ethereum Sepolia to multiple destination
chains, but it is not yet deployed against mainnet or hardened for
production use.

**Deployed contracts (testnet):**

| Chain | Network | Registry | Verifier |
|---|---|---|---|
| Base | Sepolia | `0x6C6898E6ea31E89cd65538B1EC007F8AFfD2a5CF` | `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5` |
| OP | Sepolia | `0xc8d5f2cc259cEEfB871d1FB319663B441A385CBA` | `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5` |
| Arbitrum | Sepolia | `0x6b4E1B15B85CB52425388ba7C9557de437ba8A01` | `0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5` |
| Gnosis | Chiado | -- | -- |
| Monad | Testnet | `0x3B40dd0cB126e8d521640407c6A3d663D3EAc7c5` | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| HyperEVM | Testnet | `0xd8e10a5066A4a1cd2fefec2D096E6fb8Cf2B3565` | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| Tempo | Testnet | `0x3B40dd0cB126e8d521640407c6A3d663D3EAc7c5` | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| MegaETH | Testnet | `0x3B40dd0cB126e8d521640407c6A3d663D3EAc7c5` | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| Plasma | Testnet | `0x3B40dd0cB126e8d521640407c6A3d663D3EAc7c5` | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| Starknet | Sepolia | `0x0522fb6a00ec72af9f950d6040d7bc31b86b5d4ebd372060fd02d1cebfa70ad7` | `0x79b72f62c1c6aad55c0ee0ecc68132a32db268306a19c451c35191080b7b611` |
| Solana | Devnet | `CGPJkHwUYwubDNoaLwEMMNqHcHkKz3wB3SKb2ST4i2G1` | -- |

EVM chains use the [SP1 Groth16 verifier](https://docs.succinct.xyz).
Starknet uses a [Garaga](https://www.garaga.xyz/) SP1 verifier class.
Solana verifies inside the registry program itself. Gnosis Chiado is
configured but not yet deployed.

**What works today:**

- SQLite-backed watcher and job lifecycle for root observation
- Bankai finalized-height polling and exact-block proof-bundle retrieval
- SP1 proof generation with public values and artifact handling
- Contract-side SP1 verifier and program-vkey binding
- Relay paths for EVM, Starknet Sepolia, and Solana Devnet
- Read-only API and a frontend dashboard for broadcast state

**What remains:**

- Live end-to-end validation against deployed verifier and registry
  contracts across all destination targets
- Mainnet deployment and productionization

## Repository structure

```
backend/     Runtime config, database, orchestration, and the read-only API
program/     SP1 guest program
contracts/   EVM, Starknet, and Solana destination contracts and deploy helpers
frontend/    Landing page and broadcast dashboard
```

For contract deployment, verification, and supported chain names, see
[`contracts/README.md`](contracts/README.md) and the deploy helper
`contracts/deploy.sh`.

## Running the system

Start the backend in full mode (watcher + prover + relayer + API):

```bash
cargo run -p world-id-root-replicator-backend
```

Run in API-only mode for frontend work or inspection:

```bash
cargo run -p world-id-root-replicator-backend -- --api-only
```

Manually submit a stored proof artifact to a specific chain:

```bash
cargo run -p world-id-root-replicator-backend --bin submit_proof -- \
  --chain solana-devnet \
  --artifact artifacts/proofs/job-3.bin \
  --wait
```

Pass `--registry <address>` to override the configured destination
contract for a single debugging run.

Print the current SP1 program vkey:

```bash
cargo run -p world-id-root-replicator-backend --bin print_program_vkey
```

## Docker deployment

The repository includes a Docker Compose setup that runs the Rust
backend and the static frontend together. The backend keeps its SQLite
database in a named Docker volume, and the frontend is served by Nginx
with `/api` proxied to the backend container.

The backend image is pinned to `linux/amd64` because the current SP1
builder image publishes that platform. On Apple Silicon, Docker Desktop
will run it through emulation automatically.

Start the full stack with:

```bash
docker compose up --build -d
```

The services are exposed at:

- Frontend: `http://localhost:3000`
- Backend API: `http://localhost:3001`

Compose reads the normal project `.env` file for runtime secrets and
chain configuration. It overrides only the backend bind address and
database path so the container uses:

- `LISTEN_ADDR=0.0.0.0:3001`
- `DATABASE_URL=sqlite:///data/world-id-root-replicator.db`

The named volumes are:

- `backend_db` for the SQLite database
- `backend_artifacts` for generated proof artifacts

To stop the stack:

```bash
docker compose down
```

To stop it and remove the persisted volumes as well:

```bash
docker compose down -v
```

## Links

- [Live dashboard](https://rootcast.bankai.xyz/dashboard)
- [Bankai documentation](https://docs.bankai.xyz)
- [Bankai stateless light clients](https://docs.bankai.xyz/docs/concepts/stateless-light-clients)
- [Contracts documentation](contracts/README.md)
