#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SOLANA_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
WORKSPACE_DIR=$(cd "$SOLANA_DIR/.." && pwd)
ENV_FILE="$WORKSPACE_DIR/.env"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

: "${SOLANA_DEVNET_RPC_URL:?SOLANA_DEVNET_RPC_URL must be set}"
: "${SOLANA_DEVNET_PRIVATE_KEY:?SOLANA_DEVNET_PRIVATE_KEY must be set}"

echo "Solana workspace: $SOLANA_DIR"
echo "Workspace env: $ENV_FILE"
echo "RPC URL: $SOLANA_DEVNET_RPC_URL"

if [[ "$SOLANA_DEVNET_PRIVATE_KEY" == \[* ]]; then
  echo "Wallet source: inline JSON array"
elif [[ -f "$SOLANA_DEVNET_PRIVATE_KEY" ]]; then
  echo "Wallet source: keypair file"
else
  echo "Wallet source: unsupported for deploy script"
fi

if [[ -n "${SOLANA_DEVNET_PROGRAM_ID:-}" ]]; then
  echo "Program ID: $SOLANA_DEVNET_PROGRAM_ID"
else
  echo "Program ID not set yet; deploy script will print it after deploy"
fi

if [[ -n "${PROGRAM_VKEY:-}" ]]; then
  echo "PROGRAM_VKEY configured"
else
  echo "PROGRAM_VKEY is not set"
fi

echo "Run ./script/deploy_registry.sh next"
