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
: "${SOLANA_DEVNET_PROGRAM_ID:?SOLANA_DEVNET_PROGRAM_ID must be set}"
: "${PROGRAM_VKEY:?PROGRAM_VKEY must be set}"

cd "$WORKSPACE_DIR"

cargo run -q -p world-id-root-replicator-backend --bin solana_registry_admin -- initialize
