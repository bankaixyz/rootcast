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

log() {
  printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$*"
}

usage() {
  cat <<'EOF'
Usage: deploy_registry.sh [--chain CHAIN] [--verify] [--verifier ADDRESS] [--program-vkey BYTES32]

Deploy `WorldIdRootRegistry` to an EVM testnet using the env in `../.env`.

Options:
  --chain CHAIN        Target chain: base, op, arb.
  --verify             Verify the deployed contract after deploy.
  --verifier ADDRESS   Override the SP1 verifier address.
  --program-vkey VKEY  Override the SP1 program vkey.
  --help               Show this message.

Environment:
  BASE_SEPOLIA_RPC_URL
  BASE_SEPOLIA_PRIVATE_KEY
  OP_SEPOLIA_RPC_URL
  OP_SEPOLIA_PRIVATE_KEY
  ARBITRUM_SEPOLIA_RPC_URL
  ARBITRUM_SEPOLIA_PRIVATE_KEY
  PROGRAM_VKEY            Required unless passed with --program-vkey
  ETHERSCAN_API_KEY          Required only with --verify
  SP1_VERIFIER_ADDRESS       Optional default override

Defaults:
  verifier: 0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5
  program vkey: read from PROGRAM_VKEY
EOF
}

CHAIN=base
VERIFY=0
VERIFIER_ADDRESS=${SP1_VERIFIER_ADDRESS:-0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5}
PROGRAM_VKEY=${PROGRAM_VKEY:-}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --chain)
      CHAIN=${2:?missing value for --chain}
      shift 2
      ;;
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

case "$CHAIN" in
  base|base-sepolia)
    CHAIN_NAME="Base Sepolia"
    CHAIN_ID=84532
    RPC_VAR="BASE_SEPOLIA_RPC_URL"
    KEY_VAR="BASE_SEPOLIA_PRIVATE_KEY"
    REGISTRY_VAR="BASE_SEPOLIA_REGISTRY_ADDRESS"
    VERIFY_LABEL="Basescan"
    ;;
  op|op-sepolia|optimism|optimism-sepolia)
    CHAIN_NAME="OP Sepolia"
    CHAIN_ID=11155420
    RPC_VAR="OP_SEPOLIA_RPC_URL"
    KEY_VAR="OP_SEPOLIA_PRIVATE_KEY"
    REGISTRY_VAR="OP_SEPOLIA_REGISTRY_ADDRESS"
    VERIFY_LABEL="Optimism Sepolia Etherscan"
    ;;
  arb|arbitrum|arb-sepolia|arbitrum-sepolia)
    CHAIN_NAME="Arbitrum Sepolia"
    CHAIN_ID=421614
    RPC_VAR="ARBITRUM_SEPOLIA_RPC_URL"
    KEY_VAR="ARBITRUM_SEPOLIA_PRIVATE_KEY"
    REGISTRY_VAR="ARBITRUM_SEPOLIA_REGISTRY_ADDRESS"
    VERIFY_LABEL="Arbiscan Sepolia"
    ;;
  *)
    echo "Unsupported chain: $CHAIN" >&2
    usage >&2
    exit 1
    ;;
esac

RPC_URL=${!RPC_VAR:-}
PRIVATE_KEY=${!KEY_VAR:-}

: "${RPC_URL:?$RPC_VAR must be set}"
: "${PRIVATE_KEY:?$KEY_VAR must be set}"

if [[ -z "$PROGRAM_VKEY" ]]; then
  cat >&2 <<EOF
PROGRAM_VKEY is not set.

Generate it with:
  cd "$WORKSPACE_DIR"
  cargo run -q -p world-id-root-replicator-backend --bin print_program_vkey

Then export it, for example:
  export PROGRAM_VKEY=0x...
EOF
  exit 1
fi

log "Deploying WorldIdRootRegistry to $CHAIN_NAME"
log "Using verifier: $VERIFIER_ADDRESS"
log "Using program vkey: $PROGRAM_VKEY"
log "Using rpc env: $RPC_VAR"
log "Starting forge create"

DEPLOY_OUTPUT_FILE=$(mktemp)
cleanup() {
  rm -f "$DEPLOY_OUTPUT_FILE"
}
trap cleanup EXIT

(
  cd "$CONTRACTS_DIR"
  forge create src/WorldIdRootRegistry.sol:WorldIdRootRegistry \
    --broadcast \
    --rpc-url "$RPC_URL" \
    --private-key "$PRIVATE_KEY" \
    --constructor-args "$VERIFIER_ADDRESS" "$PROGRAM_VKEY"
) | tee "$DEPLOY_OUTPUT_FILE"

DEPLOY_OUTPUT=$(cat "$DEPLOY_OUTPUT_FILE")

CONTRACT_ADDRESS=$(printf '%s\n' "$DEPLOY_OUTPUT" | sed -n 's/^Deployed to: //p' | tail -n 1)

if [[ -z "$CONTRACT_ADDRESS" ]]; then
  echo "Failed to parse deployed contract address" >&2
  exit 1
fi

echo
log "Deployed registry: $CONTRACT_ADDRESS"
log "Update .env with:"
printf '%s=%s\n' "$REGISTRY_VAR" "$CONTRACT_ADDRESS"

if [[ "$VERIFY" -eq 1 ]]; then
  : "${ETHERSCAN_API_KEY:?ETHERSCAN_API_KEY must be set when using --verify}"

  ENCODED_ARGS=$(
    cast abi-encode 'constructor(address,bytes32)' "$VERIFIER_ADDRESS" "$PROGRAM_VKEY"
  )

  echo
  log "Verifying on $VERIFY_LABEL"

  cd "$CONTRACTS_DIR"
  forge verify-contract \
    --chain "$CHAIN_ID" \
    --verifier etherscan \
    --watch \
    --compiler-version v0.8.28+commit.7893614a \
    --num-of-optimizations 200 \
    --constructor-args "$ENCODED_ARGS" \
    "$CONTRACT_ADDRESS" \
    src/WorldIdRootRegistry.sol:WorldIdRootRegistry
fi
