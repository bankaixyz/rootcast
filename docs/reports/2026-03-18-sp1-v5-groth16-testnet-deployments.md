# SP1 v5 Groth16 verifier testnet deployments

This report records the deployment status for the direct
`SP1VerifierGroth16` contract across the requested EVM-compatible testnets.

## Summary

- Deployment date: March 18, 2026
- Upstream repo: `succinctlabs/sp1-contracts`
- Upstream commit: `22c4a47cd0a388cb4e25b4f2513954e4275c74ca`
- Contract target: `contracts/src/v5.0.0/SP1VerifierGroth16.sol:SP1Verifier`
- Compiler used: `solcjs 0.8.20`
- Deployment method: `cast send --create` with raw creation bytecode
- Assumption: `gnosis` means Gnosis Chiado because the request was for testnets
- Result: 6 of 6 deployments succeeded
- Shared deployed address on every target chain:
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`

## Validation

- Live RPC `eth_chainId` checks matched all expected target chain IDs before
  deployment
- `eth_getCode` returned non-empty bytecode for the deployed address on all six
  chains
- `VERSION()` returned `"v5.0.0"` on Monad Testnet
- `VERIFIER_HASH()` returned
  `0xa4594c59bbc142f3b81c3ecb7f50a7c34bc9af7c4c444b5d48b795427e285913`
  on Monad Testnet

## Deployment matrix

| Chain | Chain ID | RPC used | Status | Contract | Explorer |
| --- | ---: | --- | --- | --- | --- |
| Monad Testnet | 10143 | `https://testnet-rpc.monad.xyz` | success | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` | `https://testnet.monadscan.com/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| HyperEVM Testnet | 998 | `https://rpc.hyperliquid-testnet.xyz/evm` | success | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` | `https://testnet.purrsec.com/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| Tempo Testnet (Moderato) | 42431 | `https://rpc.moderato.tempo.xyz` | success | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` | `https://explore.tempo.xyz/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| MegaETH Testnet | 6343 | `https://carrot.megaeth.com/rpc` | success | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` | `https://testnet-mega.etherscan.io/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| Plasma Testnet | 9746 | `https://testnet-rpc.plasma.to` | success | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` | `https://testnet.plasmascan.to/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |
| Gnosis Chiado | 10200 | `https://rpc.chiadochain.net` | success | `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` | `https://gnosis-chiado.blockscout.com/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664` |

## Chain details

### Monad Testnet

- RPC source: `https://testnet-rpc.monad.xyz`
- Preflight:
  chain ID `10143`, deployer balance `0.8 MON`, non-empty code confirmed after
  deployment
- Deployment: success
- Contract:
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Explorer:
  `https://testnet.monadscan.com/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Transaction:
  `0xf508df2923fc14a12b53b972b10821f9dbf9632ae1210d563c1fca5ba1adc74b`
- Notes:
  receipt gas used `0x153473`, effective gas price `0x1836e21000`

### HyperEVM Testnet

- RPC source: `https://rpc.hyperliquid-testnet.xyz/evm`
- Preflight:
  chain ID `998`, deployer balance `1 HYPE`, non-empty code confirmed after
  deployment
- Deployment: success
- Contract:
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Explorer:
  `https://testnet.purrsec.com/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Transaction:
  `0x7383974450d783d09ad8a32ff98a326a18e6c827f469af0f1420feba58ee08fd`
- Notes:
  the public RPC accepted the deployment transaction during this run

### Tempo Testnet (Moderato)

- RPC source: `https://rpc.moderato.tempo.xyz`
- Preflight:
  chain ID `42431`, deployer balance returned by RPC as
  `4242424242424242424242424242424242424242424242424242424242424242424242424242`,
  non-empty code confirmed after deployment
- Deployment: success
- Contract:
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Explorer:
  `https://explore.tempo.xyz/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Transaction:
  `0x0d63af8452a42370bab829ee1890253d68e534cc5210f0e6ac77847caadf31a8`
- Notes:
  Tempo uses its own fee-token model. The receipt includes
  `feeToken=0x20c0000000000000000000000000000000000000`.

### MegaETH Testnet

- RPC source: `https://carrot.megaeth.com/rpc`
- Preflight:
  chain ID `6343`, deployer balance `1 ETH`, non-empty code confirmed after
  deployment
- Deployment: success
- Contract:
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Explorer:
  `https://testnet-mega.etherscan.io/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Transaction:
  `0x7089f4df06841b315bd2498c8b159cb9d54af1476b394fb9c86a2f7d146e211e`
- Notes:
  MegaETH reported a much larger deployment gas figure than the other target
  chains, but the transaction still succeeded and code was present afterward.

### Plasma Testnet

- RPC source: `https://testnet-rpc.plasma.to`
- Preflight:
  chain ID `9746`, deployer balance `2 XPL`, non-empty code confirmed after
  deployment
- Deployment: success
- Contract:
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Explorer:
  `https://testnet.plasmascan.to/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Transaction:
  `0xecace74239253f01c5dacf49e8bf94ba9836fdad03f2bdfb569cd9ae53db898a`
- Notes:
  receipt effective gas price was `0x8`

### Gnosis Chiado

- RPC source: `https://rpc.chiadochain.net`
- Preflight:
  chain ID `10200`, deployer balance `0.001 xDAI`, non-empty code confirmed
  after deployment
- Deployment: success after fallback
- Contract:
  `0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Explorer:
  `https://gnosis-chiado.blockscout.com/address/0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664`
- Transaction:
  `0xaf68113c4cca2cb917d0fe0ec6083e5b89793e4ec22abaa9c8e6c2f6083b2c13`
- Notes:
  the first default `cast send` attempt failed during gas estimation with a
  contract-creation error. Retrying with a legacy transaction, manual gas
  limit, and `500000000` wei gas price succeeded.

## Errors and caveats

- `forge build` could not be used in this environment because it failed with
  `Operation not permitted (os error 1)`. Deployment still completed by
  compiling the verifier with `solcjs 0.8.20` and broadcasting with
  `cast send --create`.
- Explorer indexing is ecosystem-dependent. The links above are the correct
  contract-address patterns for the target explorers, but some explorers may
  lag before the contract page shows full metadata.
- Source verification was not attempted in this pass because no explorer API
  keys were provided and address-link delivery was the required output.
