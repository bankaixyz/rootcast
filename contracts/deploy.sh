#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(cd "$SCRIPT_DIR/.." && pwd)
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

CHAIN=
VERIFY=0
SKIP_BUILD=0
RESULT_LINES=()
SUPPORTED_CHAINS=(
  base-sepolia
  op-sepolia
  arbitrum-sepolia
  chiado
  monad-testnet
  hyperevm-testnet
  tempo-testnet
  megaeth-testnet
  plasma-testnet
  starknet-sepolia
  solana-devnet
)

usage() {
  cat <<'EOF'
Usage: deploy.sh [--chain CHAIN] [--verify] [--skip-build]

Deploy root registry contracts for the active destination chains.

Options:
  --chain CHAIN   Deploy a single chain using the canonical chain name from
                  ENABLED_DESTINATION_CHAINS.
  --verify        Verify EVM deployments after deploy.
  --skip-build    Skip the default rebuild step.
  --help          Show this message.
EOF
}

log() {
  printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$*"
}

warn() {
  printf '[%s] Warning: %s\n' "$(date '+%H:%M:%S')" "$*" >&2
}

is_verify_unsupported_chain() {
  case "$1" in
    monad-testnet|hyperevm-testnet|tempo-testnet|megaeth-testnet|plasma-testnet)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

contains() {
  local needle=$1
  shift
  local value
  for value in "$@"; do
    if [[ "$value" == "$needle" ]]; then
      return 0
    fi
  done
  return 1
}

is_supported_chain() {
  contains "$1" "${SUPPORTED_CHAINS[@]}"
}

is_evm_chain() {
  case "$1" in
    base-sepolia|op-sepolia|arbitrum-sepolia|chiado|monad-testnet|hyperevm-testnet|tempo-testnet|megaeth-testnet|plasma-testnet)
      return 0
      ;;
    *)
      return 1
      ;;
  esac
}

enabled_chains() {
  local raw name
  local selected=()

  if [[ -z "${ENABLED_DESTINATION_CHAINS:-}" ]]; then
    printf '%s\n' "${SUPPORTED_CHAINS[@]}"
    return
  fi

  IFS=',' read -r -a raw <<<"$ENABLED_DESTINATION_CHAINS"
  for name in "${raw[@]}"; do
    name=$(printf '%s' "$name" | xargs)
    if [[ -z "$name" ]]; then
      continue
    fi
    if ! is_supported_chain "$name"; then
      echo "Unknown destination chain in ENABLED_DESTINATION_CHAINS: $name" >&2
      exit 1
    fi
    if [[ ${#selected[@]} -eq 0 ]] || ! contains "$name" "${selected[@]}"; then
      selected+=("$name")
    fi
  done

  if [[ ${#selected[@]} -eq 0 ]]; then
    echo "ENABLED_DESTINATION_CHAINS must include at least one supported chain name" >&2
    exit 1
  fi

  printf '%s\n' "${selected[@]}"
}

load_target_chains() {
  TARGET_CHAINS=()

  if [[ -n "$CHAIN" ]]; then
    TARGET_CHAINS=("$CHAIN")
    return
  fi

  while IFS= read -r chain_name; do
    TARGET_CHAINS+=("$chain_name")
  done < <(enabled_chains)

  if [[ ${#TARGET_CHAINS[@]} -eq 0 ]]; then
    echo "No destination chains selected for deploy" >&2
    exit 1
  fi
}

run_and_capture_env_line() {
  local output_file chain env_line
  chain=$1
  shift

  output_file=$(mktemp)
  if "$@" | tee "$output_file"; then
    env_line=$(grep -E '^[A-Z0-9_]+=.+$' "$output_file" | tail -n 1 || true)
  else
    rm -f "$output_file"
    echo "Deploy failed for $chain" >&2
    exit 1
  fi
  rm -f "$output_file"

  if [[ -z "$env_line" ]]; then
    echo "Failed to capture env output for $chain" >&2
    exit 1
  fi

  RESULT_LINES+=("$env_line")
}

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
    --skip-build)
      SKIP_BUILD=1
      shift
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

if [[ -n "$CHAIN" ]] && ! is_supported_chain "$CHAIN"; then
  echo "Unsupported chain: $CHAIN" >&2
  usage >&2
  exit 1
fi

load_target_chains

NEEDS_EVM=0
NEEDS_STARKNET=0
NEEDS_SOLANA=0
for chain in "${TARGET_CHAINS[@]}"; do
  if is_evm_chain "$chain"; then
    NEEDS_EVM=1
  elif [[ "$chain" == "starknet-sepolia" ]]; then
    NEEDS_STARKNET=1
  elif [[ "$chain" == "solana-devnet" ]]; then
    NEEDS_SOLANA=1
  fi
done

if [[ "$SKIP_BUILD" -ne 1 ]]; then
  if [[ "$NEEDS_EVM" -eq 1 ]]; then
    log "Building EVM contracts"
    (
      cd "$SCRIPT_DIR/evm"
      forge build
    )
  fi

  if [[ "$NEEDS_STARKNET" -eq 1 ]]; then
    log "Building Starknet contracts"
    (
      cd "$SCRIPT_DIR/starknet"
      scarb build
    )
  fi

  if [[ "$NEEDS_SOLANA" -eq 1 ]]; then
    log "Building Solana contracts"
    (
      cd "$SCRIPT_DIR/solana"
      mkdir -p "$REPO_ROOT/.tools/cargo-home"
      CARGO_HOME="${CARGO_HOME:-$REPO_ROOT/.tools/cargo-home}" anchor build
    )
  fi
fi

for chain in "${TARGET_CHAINS[@]}"; do
  log "Deploying $chain"

  if is_evm_chain "$chain"; then
    deploy_cmd=("$SCRIPT_DIR/evm/deploy.sh" "--chain" "$chain")
    if [[ "$VERIFY" -eq 1 ]]; then
      if is_verify_unsupported_chain "$chain"; then
        warn "verification is currently unsupported for $chain; deploying without verification"
      else
        deploy_cmd+=("--verify")
      fi
    fi
    run_and_capture_env_line "$chain" "${deploy_cmd[@]}"
    continue
  fi

  if [[ "$chain" == "starknet-sepolia" ]]; then
    run_and_capture_env_line \
      "$chain" \
      "$SCRIPT_DIR/starknet/deploy.sh" \
      --chain \
      "$chain"
    continue
  fi

  run_and_capture_env_line \
    "$chain" \
    "$SCRIPT_DIR/solana/script/deploy.sh" \
    --chain \
    "$chain"
done

echo
echo "Paste these into $ENV_FILE:"
printf '%s\n' "${RESULT_LINES[@]}"
