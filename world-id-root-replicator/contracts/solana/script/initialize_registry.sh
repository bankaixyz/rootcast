#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
SOLANA_DIR=$(cd "$SCRIPT_DIR/.." && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/../../.." && pwd)
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"

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
: "${SOLANA_DEVNET_PROGRAM_ID:?SOLANA_DEVNET_PROGRAM_ID must be set}"
: "${PROGRAM_VKEY:?PROGRAM_VKEY must be set}"

cd "$REPO_ROOT"

cargo run -q -p world-id-root-replicator-backend --bin solana_registry_admin -- initialize
