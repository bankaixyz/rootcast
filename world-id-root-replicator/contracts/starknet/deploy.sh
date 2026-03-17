#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ENV_FILE="${ENV_FILE:-$REPO_ROOT/.env}"
TOOLS_DIR="${TOOLS_DIR:-$REPO_ROOT/.tools}"
REQUIRED_SNCAST_VERSION="${REQUIRED_SNCAST_VERSION:-0.57.0}"

if [ -f "$ENV_FILE" ]; then
    set -a
    # shellcheck disable=SC1090
    source "$ENV_FILE"
    set +a
fi

NETWORK="${1:-sepolia}"
ACCOUNT="${2:-${STARKNET_SEPOLIA_ACCOUNT_NAME:-world-id-root-replicator-starknet-sepolia}}"
VK="${3:-${PROGRAM_VKEY:-${WORLD_ID_ROOT_REPLICATOR_PROGRAM_VKEY:-}}}"
RPC_URL="${STARKNET_SEPOLIA_RPC_URL:-${STARKNET_SEPOLIA_RPC:-}}"
SCARB_BIN="${SCARB_BIN:-scarb}"
SNCAST_BIN="${SNCAST_BIN:-sncast}"

normalize_rpc_url() {
    local url
    url="$1"

    if [ -z "$url" ]; then
        return
    fi

    if echo "$url" | grep -q 'quiknode\.pro' && ! echo "$url" | grep -q '/rpc/v0_'; then
        url="${url%/}/rpc/v0_10"
    fi

    printf '%s\n' "$url"
}

RPC_URL="$(normalize_rpc_url "$RPC_URL")"

detect_foundry_asset() {
    local os arch
    os="$(uname -s)"
    arch="$(uname -m)"

    case "$os" in
        Darwin)
            case "$arch" in
                arm64|aarch64) echo "aarch64-apple-darwin" ;;
                x86_64) echo "x86_64-apple-darwin" ;;
                *) return 1 ;;
            esac
            ;;
        Linux)
            case "$arch" in
                arm64|aarch64) echo "aarch64-unknown-linux-gnu" ;;
                x86_64) echo "x86_64-unknown-linux-gnu" ;;
                *) return 1 ;;
            esac
            ;;
        *)
            return 1
            ;;
    esac
}

bootstrap_sncast() {
    local target platform archive_url archive_path install_dir

    platform="$(detect_foundry_asset)" || {
        echo "Error: unsupported platform for automatic sncast bootstrap" >&2
        exit 1
    }

    install_dir="$TOOLS_DIR/starknet-foundry/$REQUIRED_SNCAST_VERSION"
    target="$install_dir/bin/sncast"
    if [ -x "$target" ]; then
        SNCAST_BIN="$target"
        return
    fi

    mkdir -p "$install_dir"
    archive_path="/tmp/starknet-foundry-v$REQUIRED_SNCAST_VERSION-$platform.tar.gz"
    archive_url="https://github.com/foundry-rs/starknet-foundry/releases/download/v$REQUIRED_SNCAST_VERSION/starknet-foundry-v$REQUIRED_SNCAST_VERSION-$platform.tar.gz"

    echo "Installing sncast v$REQUIRED_SNCAST_VERSION into $install_dir..."
    curl -L --fail -o "$archive_path" "$archive_url"
    tar -xzf "$archive_path" -C "$install_dir" --strip-components=1
    SNCAST_BIN="$target"
}

ensure_sncast() {
    local version_output

    if [ "$SNCAST_BIN" != "sncast" ]; then
        return
    fi

    set +e
    version_output="$("$SNCAST_BIN" --version 2>&1)"
    set -e
    if echo "$version_output" | grep -Eq '^sncast 0\.(5[7-9]|[6-9][0-9])\.'; then
        return
    fi

    bootstrap_sncast
}

ensure_sncast

SNCAST_FALLBACK_TARGET_ARGS=(--network "$NETWORK")
SNCAST_TARGET_ARGS=("${SNCAST_FALLBACK_TARGET_ARGS[@]}")
if [ -n "$RPC_URL" ]; then
    SNCAST_TARGET_ARGS=(--url "$RPC_URL")
fi

run_sncast() {
    local output_var status_var
    output_var="$1"
    status_var="$2"
    shift 2

    local output status
    set +e
    output=$("$SNCAST_BIN" --account "$ACCOUNT" "$@" "${SNCAST_TARGET_ARGS[@]}" 2>&1)
    status=$?
    set -e

    if [ "$status" -ne 0 ] && [ -n "$RPC_URL" ] && echo "$output" | grep -Eq 'Invalid block id|uses incompatible version'; then
        echo "RPC URL $RPC_URL is incompatible with sncast, retrying with --network $NETWORK..." >&2
        set +e
        output=$("$SNCAST_BIN" --account "$ACCOUNT" "$@" "${SNCAST_FALLBACK_TARGET_ARGS[@]}" 2>&1)
        status=$?
        set -e
    fi

    printf -v "$output_var" '%s' "$output"
    printf -v "$status_var" '%s' "$status"
}

if [ -z "$ACCOUNT" ]; then
    echo "Error: Account name required"
    echo "Usage: ./deploy.sh <network> <account-name> [verification-key]"
    exit 1
fi

if [ -z "$VK" ]; then
    echo "Error: PROGRAM_VKEY or WORLD_ID_ROOT_REPLICATOR_PROGRAM_VKEY must be set"
    exit 1
fi

echo "Building contract..."
"$SCARB_BIN" --release build

echo "Declaring contract..."
run_sncast DECLARE_OUTPUT DECLARE_STATUS declare --contract-name WorldIdRootRegistry

if [ "$DECLARE_STATUS" -ne 0 ] && ! echo "$DECLARE_OUTPUT" | grep -q "is already declared"; then
    echo "Declare failed"
    echo "$DECLARE_OUTPUT"
    exit 1
fi

if echo "$DECLARE_OUTPUT" | grep -q "is already declared"; then
    echo "Contract already declared, using existing class hash"
    CLASS_HASH=$(echo "$DECLARE_OUTPUT" | grep -o '0x[0-9a-fA-F]*' | head -1)
else
    CLASS_HASH=$(echo "$DECLARE_OUTPUT" | sed -nE 's/^(class_hash:|Class Hash:)[[:space:]]*(0x[0-9a-fA-F]+)$/\2/p' | head -1)
fi

if [ -z "$CLASS_HASH" ]; then
    echo "Failed to get class hash"
    echo "$DECLARE_OUTPUT"
    exit 1
fi

echo "Deploying contract..."
VK_STRIPPED=$(echo "$VK" | sed 's/0x//')
VK_PADDED=$(printf "%064s" "$VK_STRIPPED" | tr ' ' '0')
VK_HIGH=$(echo "$VK_PADDED" | cut -c1-32)
VK_LOW=$(echo "$VK_PADDED" | cut -c33-64)
echo "VK (u256): high=0x$VK_HIGH, low=0x$VK_LOW"
run_sncast DEPLOY_OUTPUT DEPLOY_STATUS deploy --class-hash "$CLASS_HASH" --constructor-calldata 0x$VK_LOW 0x$VK_HIGH
CONTRACT_ADDRESS=$(echo "$DEPLOY_OUTPUT" | sed -nE 's/^(contract_address:|Contract Address:)[[:space:]]*(0x[0-9a-fA-F]+)$/\2/p' | head -1)

if [ "$DEPLOY_STATUS" -ne 0 ] || [ -z "$CONTRACT_ADDRESS" ]; then
    echo "Deployment failed"
    echo "$DEPLOY_OUTPUT"
    exit 1
fi

echo ""
echo "Deployed successfully!"
echo "Contract: $CONTRACT_ADDRESS"
echo "Class:    $CLASS_HASH"
echo "Explorer: https://sepolia.voyager.online/contract/$CONTRACT_ADDRESS"
echo ""
echo "Add this to your .env:"
echo "STARKNET_SEPOLIA_REGISTRY_ADDRESS=$CONTRACT_ADDRESS"
