#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CONTRACTS_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
WORKSPACE_DIR=$(cd "$CONTRACTS_DIR/.." && pwd)
ENV_FILE="$WORKSPACE_DIR/.env"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

usage() {
  cat <<'EOF'
Usage: deploy_base_sepolia_registry.sh [--verify] [--verifier ADDRESS] [--program-vkey BYTES32]

Deploy `WorldIdRootRegistry` to Base Sepolia using the env in `../.env`.

Options:
  --verify             Verify the deployed contract on Basescan after deploy.
  --verifier ADDRESS   Override the SP1 verifier address.
  --program-vkey VKEY  Override the SP1 program vkey.
  --help               Show this message.

Environment:
  BASE_SEPOLIA_RPC_URL
  BASE_SEPOLIA_PRIVATE_KEY
  ETHERSCAN_API_KEY          Required only with --verify
  SP1_VERIFIER_ADDRESS       Optional default override
  SP1_PROGRAM_VKEY           Optional default override

Defaults:
  verifier: 0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5
  program vkey: derived by running the print_program_vkey helper
EOF
}

VERIFY=0
VERIFIER_ADDRESS=${SP1_VERIFIER_ADDRESS:-0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5}
PROGRAM_VKEY=${SP1_PROGRAM_VKEY:-}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --verify)
      VERIFY=1
      shift
      ;;
    --verifier)
      VERIFIER_ADDRESS=${2:?missing value for --verifier}
      shift 2
      ;;
    --program-vkey)
      PROGRAM_VKEY=${2:?missing value for --program-vkey}
      shift 2
      ;;
    --help|-h)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

: "${BASE_SEPOLIA_RPC_URL:?BASE_SEPOLIA_RPC_URL must be set}"
: "${BASE_SEPOLIA_PRIVATE_KEY:?BASE_SEPOLIA_PRIVATE_KEY must be set}"

if [[ -z "$PROGRAM_VKEY" ]]; then
  PROGRAM_VKEY=$(
    cd "$WORKSPACE_DIR"
    cargo run -q -p world-id-root-replicator-backend --bin print_program_vkey
  )
fi

echo "Deploying WorldIdRootRegistry"
echo "  verifier:     $VERIFIER_ADDRESS"
echo "  program vkey: $PROGRAM_VKEY"

DEPLOY_OUTPUT=$(
  cd "$CONTRACTS_DIR"
  forge create src/WorldIdRootRegistry.sol:WorldIdRootRegistry \
    --broadcast \
    --rpc-url "$BASE_SEPOLIA_RPC_URL" \
    --private-key "$BASE_SEPOLIA_PRIVATE_KEY" \
    --constructor-args "$VERIFIER_ADDRESS" "$PROGRAM_VKEY"
)

printf '%s\n' "$DEPLOY_OUTPUT"

CONTRACT_ADDRESS=$(printf '%s\n' "$DEPLOY_OUTPUT" | sed -n 's/^Deployed to: //p' | tail -n 1)

if [[ -z "$CONTRACT_ADDRESS" ]]; then
  echo "Failed to parse deployed contract address" >&2
  exit 1
fi

echo
echo "Deployed registry: $CONTRACT_ADDRESS"
echo "Update .env with:"
echo "BASE_SEPOLIA_REGISTRY_ADDRESS=$CONTRACT_ADDRESS"

if [[ "$VERIFY" -eq 1 ]]; then
  : "${ETHERSCAN_API_KEY:?ETHERSCAN_API_KEY must be set when using --verify}"

  ENCODED_ARGS=$(
    cast abi-encode 'constructor(address,bytes32)' "$VERIFIER_ADDRESS" "$PROGRAM_VKEY"
  )

  echo
  echo "Verifying on Basescan..."

  cd "$CONTRACTS_DIR"
  forge verify-contract \
    --chain base-sepolia \
    --verifier etherscan \
    --watch \
    --compiler-version v0.8.28+commit.7893614a \
    --num-of-optimizations 200 \
    --constructor-args "$ENCODED_ARGS" \
    "$CONTRACT_ADDRESS" \
    src/WorldIdRootRegistry.sol:WorldIdRootRegistry
fi
