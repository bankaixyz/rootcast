#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"
ACCOUNT_NAME="${STARKNET_SEPOLIA_ACCOUNT_NAME:-world-id-root-replicator-starknet-sepolia}"
PROFILE_NAME="${STARKNET_SEPOLIA_PROFILE_NAME:-starknet-sepolia}"

require_command() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "missing required command: $1" >&2
    exit 1
  fi
}

require_env() {
  local name="$1"
  if [ -z "${!name:-}" ]; then
    echo "missing required env var: $name" >&2
    exit 1
  fi
}

parse_args() {
  while [ $# -gt 0 ]; do
    case "$1" in
      --program-vkey)
        PROGRAM_VKEY="$2"
        shift 2
        ;;
      *)
        echo "unknown argument: $1" >&2
        echo "usage: ./deploy.sh [--program-vkey 0x...]" >&2
        exit 1
        ;;
    esac
  done
}

load_env() {
  if [ -f "$ENV_FILE" ]; then
    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE"
    set +a
  fi
}

split_u256() {
  local value="${1#0x}"
  value="$(printf "%064s" "$value" | tr ' ' '0')"
  VK_HIGH="0x${value:0:32}"
  VK_LOW="0x${value:32:32}"
}

ensure_account() {
  local output
  if output="$(
    sncast account import \
      --name "$ACCOUNT_NAME" \
      --address "$STARKNET_SEPOLIA_ACCOUNT_ADDRESS" \
      --private-key "$STARKNET_SEPOLIA_PRIVATE_KEY" \
      --type oz \
      --url "$STARKNET_SEPOLIA_RPC_URL" \
      --add-profile "$PROFILE_NAME" 2>&1
  )"; then
    printf '%s\n' "$output"
    return
  fi

  if printf '%s\n' "$output" | rg -q "already exists|Account with name"; then
    printf '%s\n' "$output"
    return
  fi

  printf '%s\n' "$output" >&2
  exit 1
}

declare_contract() {
  local output
  output="$(
    sncast --profile "$PROFILE_NAME" --account "$ACCOUNT_NAME" declare --contract-name WorldIdRootRegistry 2>&1
  )"

  if printf '%s\n' "$output" | rg -q "already declared"; then
    CLASS_HASH="$(printf '%s\n' "$output" | rg -o '0x[0-9a-fA-F]+' | head -1)"
  else
    CLASS_HASH="$(printf '%s\n' "$output" | sed -n 's/^class_hash: //p' | head -1)"
  fi

  if [ -z "${CLASS_HASH:-}" ]; then
    printf '%s\n' "$output" >&2
    exit 1
  fi
}

deploy_contract() {
  local output
  output="$(
    sncast --profile "$PROFILE_NAME" --account "$ACCOUNT_NAME" deploy \
      --class-hash "$CLASS_HASH" \
      --constructor-calldata "$VK_LOW" "$VK_HIGH" 2>&1
  )"

  CONTRACT_ADDRESS="$(printf '%s\n' "$output" | sed -n 's/^contract_address: //p' | head -1)"

  if [ -z "${CONTRACT_ADDRESS:-}" ]; then
    printf '%s\n' "$output" >&2
    exit 1
  fi
}

main() {
  parse_args "$@"
  load_env

  require_command scarb
  require_command sncast
  require_command rg

  export STARKNET_SEPOLIA_RPC_URL="${STARKNET_SEPOLIA_RPC_URL:-${STARKNET_SEPOLIA_RPC:-}}"
  export STARKNET_SEPOLIA_PRIVATE_KEY="${STARKNET_SEPOLIA_PRIVATE_KEY:-${STARKNET_PRIVATE_KEY:-}}"
  export STARKNET_SEPOLIA_ACCOUNT_ADDRESS="${STARKNET_SEPOLIA_ACCOUNT_ADDRESS:-${STARKNET_ACCOUNT_ADDRESS:-}}"
  PROGRAM_VKEY="${PROGRAM_VKEY:-${WORLD_ID_ROOT_REPLICATOR_PROGRAM_VKEY:-}}"

  require_env STARKNET_SEPOLIA_RPC_URL
  require_env STARKNET_SEPOLIA_PRIVATE_KEY
  require_env STARKNET_SEPOLIA_ACCOUNT_ADDRESS
  require_env PROGRAM_VKEY

  split_u256 "$PROGRAM_VKEY"

  cd "$SCRIPT_DIR"
  scarb build
  ensure_account
  declare_contract
  deploy_contract

  echo
  echo "Starknet Sepolia deployment complete"
  echo "class hash:        $CLASS_HASH"
  echo "contract address:  $CONTRACT_ADDRESS"
  echo "explorer:          https://sepolia.starkscan.co/contract/$CONTRACT_ADDRESS"
  echo
  echo "Add this to your .env:"
  echo "STARKNET_SEPOLIA_REGISTRY_ADDRESS=$CONTRACT_ADDRESS"
}

main "$@"
