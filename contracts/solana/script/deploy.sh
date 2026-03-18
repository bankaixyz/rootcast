#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SOLANA_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/../../.." && pwd)
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"
PROGRAM_ARTIFACT_BASENAME="world_id_root_registry_solana"
CHAIN="solana-devnet"

usage() {
  cat <<'EOF'
Usage: deploy.sh [--chain solana-devnet]

Deploy the Solana Devnet program and initialize the registry PDA.
EOF
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --chain)
      CHAIN=${2:?missing value for --chain}
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

if [[ "$CHAIN" != "solana-devnet" ]]; then
  echo "Unsupported chain: $CHAIN" >&2
  usage >&2
  exit 1
fi

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

export CARGO_HOME="${CARGO_HOME:-$REPO_ROOT/.tools/cargo-home}"
mkdir -p "$CARGO_HOME"

: "${SOLANA_DEVNET_RPC_URL:?SOLANA_DEVNET_RPC_URL must be set}"
: "${SOLANA_DEVNET_PRIVATE_KEY:?SOLANA_DEVNET_PRIVATE_KEY must be set}"
: "${PROGRAM_VKEY:?PROGRAM_VKEY must be set}"

wallet_file=""
cleanup() {
  if [[ -n "$wallet_file" ]]; then
    rm -f "$wallet_file"
  fi
}
trap cleanup EXIT

wallet_path() {
  if [[ "$SOLANA_DEVNET_PRIVATE_KEY" == \[* ]]; then
    wallet_file=$(mktemp)
    printf '%s' "$SOLANA_DEVNET_PRIVATE_KEY" > "$wallet_file"
    printf '%s\n' "$wallet_file"
    return
  fi

  if [[ -f "$SOLANA_DEVNET_PRIVATE_KEY" ]]; then
    printf '%s\n' "$SOLANA_DEVNET_PRIVATE_KEY"
    return
  fi

  echo "deploy script expects SOLANA_DEVNET_PRIVATE_KEY as a JSON array or keypair file path" >&2
  exit 1
}

wallet_path=$(wallet_path)

cd "$SOLANA_DIR"
anchor keys sync
anchor deploy \
  --provider.cluster "$SOLANA_DEVNET_RPC_URL" \
  --provider.wallet "$wallet_path"

program_keypair="target/deploy/${PROGRAM_ARTIFACT_BASENAME}-keypair.json"
program_id=$(solana address -k "$program_keypair")
export SOLANA_DEVNET_PROGRAM_ID="$program_id"

init_signature=$("$SCRIPT_DIR/initialize_registry.sh")
inspect_output=$(
  cd "$REPO_ROOT"
  cargo run -q -p world-id-root-replicator-backend --bin solana_registry_admin -- inspect
)
state_pda=$(printf '%s\n' "$inspect_output" | sed -n 's/^state_pda=//p' | tail -n 1)

echo
echo "Deployed Solana program:"
echo "SOLANA_DEVNET_PROGRAM_ID=$program_id"
echo
echo "Initialized Solana registry:"
echo "State PDA: $state_pda"
echo "Initialize signature: $init_signature"
