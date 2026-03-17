#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SOLANA_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
WORKSPACE_DIR=$(cd "$SOLANA_DIR/.." && pwd)
ENV_FILE="$WORKSPACE_DIR/.env"
PROGRAM_ARTIFACT_BASENAME="world_id_root_registry_solana"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

: "${SOLANA_DEVNET_RPC_URL:?SOLANA_DEVNET_RPC_URL must be set}"
: "${SOLANA_DEVNET_PRIVATE_KEY:?SOLANA_DEVNET_PRIVATE_KEY must be set}"

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
anchor build
anchor keys sync
anchor deploy \
  --provider.cluster "$SOLANA_DEVNET_RPC_URL" \
  --provider.wallet "$wallet_path"

program_keypair="target/deploy/${PROGRAM_ARTIFACT_BASENAME}-keypair.json"
program_id=$(solana address -k "$program_keypair")

echo
echo "Deployed Solana program:"
echo "SOLANA_DEVNET_PROGRAM_ID=$program_id"
echo
echo "Next step:"
echo "  ./script/initialize_registry.sh"
