#!/usr/bin/env bash

set -euo pipefail

SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
CONTRACTS_DIR="$SCRIPT_DIR"
REPO_ROOT=$(cd "$SCRIPT_DIR/../.." && pwd)
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"

if [[ -f "$ENV_FILE" ]]; then
  set -a
  # shellcheck disable=SC1090
  source "$ENV_FILE"
  set +a
fi

log() {
  printf '[%s] %s\n' "$(date '+%H:%M:%S')" "$*"
}

warn() {
  printf '[%s] Warning: %s\n' "$(date '+%H:%M:%S')" "$*" >&2
}

usage() {
  cat <<'EOF'
Usage: deploy.sh [--chain CHAIN] [--verify] [--address ADDRESS] [--verifier ADDRESS] [--program-vkey BYTES32]

Deploy `WorldIdRootRegistry` to one EVM destination chain.

Options:
  --chain CHAIN        Target chain: base-sepolia, op-sepolia, arbitrum-sepolia,
                       chiado, monad-testnet, hyperevm-testnet, tempo-testnet,
                       megaeth-testnet, or plasma-testnet.
  --verify             Verify the deployed contract after deploy.
  --address ADDRESS    Use an existing deployment address. Skips deploy and
                       verifies that address.
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
  CHIADO_RPC_URL
  CHIADO_PRIVATE_KEY
  MONAD_TESTNET_RPC_URL
  MONAD_TESTNET_PRIVATE_KEY
  HYPEREVM_TESTNET_RPC_URL
  HYPEREVM_TESTNET_PRIVATE_KEY
  TEMPO_TESTNET_RPC_URL
  TEMPO_TESTNET_PRIVATE_KEY
  MEGAETH_TESTNET_RPC_URL
  MEGAETH_TESTNET_PRIVATE_KEY
  PLASMA_TESTNET_RPC_URL
  PLASMA_TESTNET_PRIVATE_KEY
  PROGRAM_VKEY
  ETHERSCAN_API_KEY
  PLASMA_TESTNET_API_KEY
  CHIADO_SP1_VERIFIER_ADDRESS
  MONAD_TESTNET_SP1_VERIFIER_ADDRESS
  HYPEREVM_TESTNET_SP1_VERIFIER_ADDRESS
  TEMPO_TESTNET_SP1_VERIFIER_ADDRESS
  MEGAETH_TESTNET_SP1_VERIFIER_ADDRESS
  PLASMA_TESTNET_SP1_VERIFIER_ADDRESS
  SP1_VERIFIER_ADDRESS
EOF
}

CHAIN=base-sepolia
VERIFY=0
EXISTING_ADDRESS=
VERIFIER_ADDRESS=
PROGRAM_VKEY=${PROGRAM_VKEY:-}
DEFAULT_V5_GROTH16_VERIFIER=0x50ACFBEdecf4cbe350E1a86fC6f03a821772f1e5
DEFAULT_CUSTOM_TESTNET_VERIFIER=0x9e630e6A6BFbcF1b1c213552Aaea5469ff5C9664

first_set_env() {
  local name
  for name in "$@"; do
    if [[ -n "${!name:-}" ]]; then
      printf '%s' "${!name}"
      return 0
    fi
  done
  return 1
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
    --address)
      EXISTING_ADDRESS=${2:?missing value for --address}
      shift 2
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

if [[ -n "$EXISTING_ADDRESS" && "$VERIFY" -ne 1 ]]; then
  echo "--address only makes sense together with --verify" >&2
  exit 1
fi

VERIFY_URL=

case "$CHAIN" in
  base-sepolia)
    CHAIN_NAME="Base Sepolia"
    CHAIN_ID=84532
    RPC_VAR="BASE_SEPOLIA_RPC_URL"
    KEY_VAR="BASE_SEPOLIA_PRIVATE_KEY"
    REGISTRY_VAR="BASE_SEPOLIA_REGISTRY_ADDRESS"
    VERIFY_LABEL="Basescan"
    VERIFY_KIND="etherscan"
    VERIFY_API_KEY_REQUIRED=1
    VERIFY_API_KEY_VARS=(ETHERSCAN_API_KEY)
    ;;
  op-sepolia)
    CHAIN_NAME="OP Sepolia"
    CHAIN_ID=11155420
    RPC_VAR="OP_SEPOLIA_RPC_URL"
    KEY_VAR="OP_SEPOLIA_PRIVATE_KEY"
    REGISTRY_VAR="OP_SEPOLIA_REGISTRY_ADDRESS"
    VERIFY_LABEL="Optimism Sepolia Etherscan"
    VERIFY_KIND="etherscan"
    VERIFY_API_KEY_REQUIRED=1
    VERIFY_API_KEY_VARS=(ETHERSCAN_API_KEY)
    ;;
  arbitrum-sepolia)
    CHAIN_NAME="Arbitrum Sepolia"
    CHAIN_ID=421614
    RPC_VAR="ARBITRUM_SEPOLIA_RPC_URL"
    KEY_VAR="ARBITRUM_SEPOLIA_PRIVATE_KEY"
    REGISTRY_VAR="ARBITRUM_SEPOLIA_REGISTRY_ADDRESS"
    VERIFY_LABEL="Arbiscan Sepolia"
    VERIFY_KIND="etherscan"
    VERIFY_API_KEY_REQUIRED=1
    VERIFY_API_KEY_VARS=(ETHERSCAN_API_KEY)
    ;;
  chiado)
    CHAIN_NAME="Gnosis Chiado"
    CHAIN_ID=10200
    RPC_VAR="CHIADO_RPC_URL"
    KEY_VAR="CHIADO_PRIVATE_KEY"
    REGISTRY_VAR="CHIADO_REGISTRY_ADDRESS"
    VERIFY_LABEL="Chiado Blockscout"
    VERIFY_KIND="blockscout"
    VERIFY_URL="https://gnosis-chiado.blockscout.com/api/"
    VERIFY_API_KEY_REQUIRED=0
    VERIFY_API_KEY_VARS=()
    ;;
  monad-testnet)
    CHAIN_NAME="Monad Testnet"
    CHAIN_ID=10143
    RPC_VAR="MONAD_TESTNET_RPC_URL"
    KEY_VAR="MONAD_TESTNET_PRIVATE_KEY"
    REGISTRY_VAR="MONAD_TESTNET_REGISTRY_ADDRESS"
    VERIFY_LABEL="Sourcify"
    VERIFY_KIND="sourcify"
    VERIFY_URL="https://sourcify.dev/server"
    VERIFY_API_KEY_REQUIRED=0
    VERIFY_API_KEY_VARS=()
    ;;
  hyperevm-testnet)
    CHAIN_NAME="HyperEVM Testnet"
    CHAIN_ID=998
    RPC_VAR="HYPEREVM_TESTNET_RPC_URL"
    KEY_VAR="HYPEREVM_TESTNET_PRIVATE_KEY"
    REGISTRY_VAR="HYPEREVM_TESTNET_REGISTRY_ADDRESS"
    VERIFY_LABEL="Purrsec"
    VERIFY_KIND="sourcify"
    VERIFY_URL="https://sourcify.parsec.finance/verify"
    VERIFY_API_KEY_REQUIRED=0
    VERIFY_API_KEY_VARS=(ETHERSCAN_API_KEY)
    ;;
  tempo-testnet)
    CHAIN_NAME="Tempo Testnet"
    CHAIN_ID=42431
    RPC_VAR="TEMPO_TESTNET_RPC_URL"
    KEY_VAR="TEMPO_TESTNET_PRIVATE_KEY"
    REGISTRY_VAR="TEMPO_TESTNET_REGISTRY_ADDRESS"
    VERIFY_LABEL="Tempo contracts verifier"
    VERIFY_KIND="sourcify"
    VERIFY_URL="https://contracts.tempo.xyz"
    VERIFY_API_KEY_REQUIRED=0
    VERIFY_API_KEY_VARS=(ETHERSCAN_API_KEY)
    ;;
  megaeth-testnet)
    CHAIN_NAME="MegaETH Testnet"
    CHAIN_ID=6343
    RPC_VAR="MEGAETH_TESTNET_RPC_URL"
    KEY_VAR="MEGAETH_TESTNET_PRIVATE_KEY"
    REGISTRY_VAR="MEGAETH_TESTNET_REGISTRY_ADDRESS"
    VERIFY_LABEL="MegaETH Etherscan"
    VERIFY_KIND="custom"
    VERIFY_URL="https://testnet-mega.etherscan.io/api"
    VERIFY_API_KEY_REQUIRED=1
    VERIFY_API_KEY_VARS=(ETHERSCAN_API_KEY)
    ;;
  plasma-testnet)
    CHAIN_NAME="Plasma Testnet"
    CHAIN_ID=9746
    RPC_VAR="PLASMA_TESTNET_RPC_URL"
    KEY_VAR="PLASMA_TESTNET_PRIVATE_KEY"
    REGISTRY_VAR="PLASMA_TESTNET_REGISTRY_ADDRESS"
    VERIFY_LABEL="PlasmaScan"
    VERIFY_KIND="etherscan"
    VERIFY_URL="https://testnet.plasmascan.to/api"
    VERIFY_API_KEY_REQUIRED=1
    VERIFY_API_KEY_VARS=(ETHERSCAN_API_KEY)
    ;;
  *)
    echo "Unsupported chain: $CHAIN" >&2
    usage >&2
    exit 1
    ;;
esac

RPC_URL=${!RPC_VAR:-}
: "${RPC_URL:?$RPC_VAR must be set}"

if [[ -z "$EXISTING_ADDRESS" ]]; then
  PRIVATE_KEY=${!KEY_VAR:-}
  : "${PRIVATE_KEY:?$KEY_VAR must be set}"
fi

if [[ -z "$VERIFIER_ADDRESS" ]]; then
  if [[ "$CHAIN_ID" -eq 10200 ]]; then
    VERIFIER_ADDRESS=${CHIADO_SP1_VERIFIER_ADDRESS:-${SP1_VERIFIER_ADDRESS:-}}
  elif [[ "$CHAIN_ID" -eq 10143 ]]; then
    VERIFIER_ADDRESS=${MONAD_TESTNET_SP1_VERIFIER_ADDRESS:-$DEFAULT_CUSTOM_TESTNET_VERIFIER}
  elif [[ "$CHAIN_ID" -eq 998 ]]; then
    VERIFIER_ADDRESS=${HYPEREVM_TESTNET_SP1_VERIFIER_ADDRESS:-$DEFAULT_CUSTOM_TESTNET_VERIFIER}
  elif [[ "$CHAIN_ID" -eq 42431 ]]; then
    VERIFIER_ADDRESS=${TEMPO_TESTNET_SP1_VERIFIER_ADDRESS:-$DEFAULT_CUSTOM_TESTNET_VERIFIER}
  elif [[ "$CHAIN_ID" -eq 6343 ]]; then
    VERIFIER_ADDRESS=${MEGAETH_TESTNET_SP1_VERIFIER_ADDRESS:-$DEFAULT_CUSTOM_TESTNET_VERIFIER}
  elif [[ "$CHAIN_ID" -eq 9746 ]]; then
    VERIFIER_ADDRESS=${PLASMA_TESTNET_SP1_VERIFIER_ADDRESS:-$DEFAULT_CUSTOM_TESTNET_VERIFIER}
  else
    VERIFIER_ADDRESS=${SP1_VERIFIER_ADDRESS:-$DEFAULT_V5_GROTH16_VERIFIER}
  fi
fi

if [[ -z "$VERIFIER_ADDRESS" ]]; then
  cat >&2 <<EOF
SP1 verifier address is not set for $CHAIN_NAME.

Provide it with one of:
  --verifier 0x...
  export CHIADO_SP1_VERIFIER_ADDRESS=0x...
  export MONAD_TESTNET_SP1_VERIFIER_ADDRESS=0x...
  export HYPEREVM_TESTNET_SP1_VERIFIER_ADDRESS=0x...
  export TEMPO_TESTNET_SP1_VERIFIER_ADDRESS=0x...
  export MEGAETH_TESTNET_SP1_VERIFIER_ADDRESS=0x...
  export PLASMA_TESTNET_SP1_VERIFIER_ADDRESS=0x...
  export SP1_VERIFIER_ADDRESS=0x...

Succinct's published deployments do not currently include chain id $CHAIN_ID,
so this script cannot choose a safe default verifier for you.
EOF
  exit 1
fi

if [[ -z "$PROGRAM_VKEY" ]]; then
  cat >&2 <<EOF
PROGRAM_VKEY is not set.

Generate it with:
  cd "$REPO_ROOT"
  cargo run -q -p world-id-root-replicator-backend --bin print_program_vkey

Then export it, for example:
  export PROGRAM_VKEY=0x...
EOF
  exit 1
fi

if [[ -n "$EXISTING_ADDRESS" ]]; then
  CONTRACT_ADDRESS="$EXISTING_ADDRESS"
  log "Skipping deploy and using existing registry: $CONTRACT_ADDRESS"
else
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
fi

if [[ "$VERIFY" -eq 1 ]]; then
  case "$CHAIN" in
    monad-testnet|hyperevm-testnet|tempo-testnet|megaeth-testnet|plasma-testnet)
      warn "verification is currently unsupported for $CHAIN; skipping verification"
      exit 0
      ;;
  esac

  VERIFY_API_KEY=
  if [[ ${#VERIFY_API_KEY_VARS[@]} -gt 0 ]]; then
    VERIFY_API_KEY=$(first_set_env "${VERIFY_API_KEY_VARS[@]}" || true)
  fi

  if [[ "${VERIFY_API_KEY_REQUIRED:-0}" -eq 1 && -z "$VERIFY_API_KEY" ]]; then
    echo "Verification on $VERIFY_LABEL requires one of:" >&2
    printf '  %s\n' "${VERIFY_API_KEY_VARS[@]}" >&2
    exit 1
  fi

  ENCODED_ARGS=$(
    cast abi-encode 'constructor(address,bytes32)' "$VERIFIER_ADDRESS" "$PROGRAM_VKEY"
  )

  echo
  log "Verifying on $VERIFY_LABEL"

  cd "$CONTRACTS_DIR"
  VERIFY_CMD=(
    forge verify-contract
    --chain "$CHAIN_ID"
    --rpc-url "$RPC_URL"
    --watch
    --compiler-version v0.8.28+commit.7893614a
    --num-of-optimizations 200
    --constructor-args "$ENCODED_ARGS"
  )

  if [[ "$VERIFY_KIND" == "etherscan" ]]; then
    VERIFY_CMD+=(--verifier etherscan)
    if [[ -n "$VERIFY_URL" ]]; then
      VERIFY_CMD+=(--verifier-url "$VERIFY_URL")
    fi
    if [[ -n "$VERIFY_API_KEY" ]]; then
      VERIFY_CMD+=(--etherscan-api-key "$VERIFY_API_KEY")
    fi
  elif [[ "$VERIFY_KIND" == "blockscout" ]]; then
    VERIFY_CMD+=(--verifier blockscout --verifier-url "$VERIFY_URL")
  elif [[ "$VERIFY_KIND" == "sourcify" ]]; then
    VERIFY_CMD+=(--verifier sourcify --verifier-url "$VERIFY_URL")
  elif [[ "$VERIFY_KIND" == "custom" ]]; then
    VERIFY_CMD+=(--verifier custom --verifier-url "$VERIFY_URL")
    if [[ -n "$VERIFY_API_KEY" ]]; then
      VERIFY_CMD+=(--verifier-api-key "$VERIFY_API_KEY")
    fi
  else
    echo "Verification is not configured yet for $CHAIN_NAME." >&2
    exit 1
  fi

  VERIFY_CMD+=(
    "$CONTRACT_ADDRESS"
    src/WorldIdRootRegistry.sol:WorldIdRootRegistry
  )

  "${VERIFY_CMD[@]}"
fi
