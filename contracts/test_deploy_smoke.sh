#!/usr/bin/env bash

set -euo pipefail

REPO_ROOT=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
STUB_BIN=$(mktemp -d)
TMP_DIR=$(mktemp -d)
LOG_FILE="$TMP_DIR/tool.log"
ENV_FILE="$TMP_DIR/test.env"

cleanup() {
  rm -rf "$STUB_BIN" "$TMP_DIR"
}
trap cleanup EXIT

cat >"$STUB_BIN/forge" <<'EOF'
#!/usr/bin/env bash
echo "forge $*" >>"$LOG_FILE"
case "$1" in
  build)
    ;;
  create)
    echo "Deployed to: 0x1111111111111111111111111111111111111111"
    ;;
  verify-contract)
    echo "verified"
    ;;
esac
EOF

cat >"$STUB_BIN/scarb" <<'EOF'
#!/usr/bin/env bash
echo "scarb $*" >>"$LOG_FILE"
EOF

cat >"$STUB_BIN/sncast" <<'EOF'
#!/usr/bin/env bash
echo "sncast $*" >>"$LOG_FILE"
if [[ "$1" == "--version" ]]; then
  echo "sncast 0.57.0"
elif printf '%s\n' "$*" | grep -q "declare"; then
  echo "Class Hash: 0xabc123"
else
  echo "Contract Address: 0x0456"
fi
EOF

cat >"$STUB_BIN/anchor" <<'EOF'
#!/usr/bin/env bash
echo "anchor $*" >>"$LOG_FILE"
EOF

cat >"$STUB_BIN/solana" <<'EOF'
#!/usr/bin/env bash
echo "solana $*" >>"$LOG_FILE"
echo "So11111111111111111111111111111111111111112"
EOF

cat >"$STUB_BIN/cargo" <<'EOF'
#!/usr/bin/env bash
echo "cargo $*" >>"$LOG_FILE"
if printf '%s\n' "$*" | grep -q "inspect"; then
  echo "program_id=So11111111111111111111111111111111111111112"
  echo "state_pda=State1111111111111111111111111111111111111"
else
  echo "InitSig111111111111111111111111111111111111111"
fi
EOF

chmod +x \
  "$STUB_BIN/forge" \
  "$STUB_BIN/scarb" \
  "$STUB_BIN/sncast" \
  "$STUB_BIN/anchor" \
  "$STUB_BIN/solana" \
  "$STUB_BIN/cargo"

cat >"$ENV_FILE" <<'EOF'
ENABLED_DESTINATION_CHAINS=base-sepolia,starknet-sepolia,solana-devnet
PROGRAM_VKEY=0x00121643a8e0b1426431683ed5bce193445f3c596ad02d126103658502d6af3f
BASE_SEPOLIA_RPC_URL=https://example.invalid/base
BASE_SEPOLIA_PRIVATE_KEY=0xabc
STARKNET_SEPOLIA_RPC_URL=https://example.invalid/starknet
STARKNET_SEPOLIA_PRIVATE_KEY=0xabc
STARKNET_SEPOLIA_ACCOUNT_ADDRESS=0x123
SOLANA_DEVNET_RPC_URL=https://example.invalid/solana
SOLANA_DEVNET_PRIVATE_KEY=[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1]
EOF

export LOG_FILE
export PATH="$STUB_BIN:$PATH"

run_output=$(
  ENV_FILE="$ENV_FILE" "$REPO_ROOT/contracts/deploy.sh"
)
printf '%s\n' "$run_output" | grep -q '^BASE_SEPOLIA_REGISTRY_ADDRESS='
printf '%s\n' "$run_output" | grep -q '^STARKNET_SEPOLIA_REGISTRY_ADDRESS='
printf '%s\n' "$run_output" | grep -q '^SOLANA_DEVNET_PROGRAM_ID='
grep -q 'forge build' "$LOG_FILE"
grep -q 'scarb build' "$LOG_FILE"
grep -q 'anchor build' "$LOG_FILE"
grep -q 'sncast --account .* declare' "$LOG_FILE"
grep -q 'cargo run -q -p world-id-root-replicator-backend --bin solana_registry_admin -- initialize' "$LOG_FILE"

: >"$LOG_FILE"
ENV_FILE="$ENV_FILE" "$REPO_ROOT/contracts/deploy.sh" --skip-build --chain solana-devnet >/dev/null
if grep -q 'anchor build' "$LOG_FILE"; then
  echo "--skip-build should not run anchor build" >&2
  exit 1
fi

if ENV_FILE="$ENV_FILE" "$REPO_ROOT/contracts/deploy.sh" --chain monad >/dev/null 2>&1; then
  echo "alias chain names should be rejected" >&2
  exit 1
fi

for chain in monad-testnet hyperevm-testnet tempo-testnet megaeth-testnet plasma-testnet; do
  upper_chain=$(printf '%s' "$chain" | tr '[:lower:]-' '[:upper:]_')
  cat >"$ENV_FILE" <<EOF
ENABLED_DESTINATION_CHAINS=$chain
PROGRAM_VKEY=0x00121643a8e0b1426431683ed5bce193445f3c596ad02d126103658502d6af3f
${upper_chain}_RPC_URL=https://example.invalid/$chain
${upper_chain}_PRIVATE_KEY=0xabc
EOF

  : >"$LOG_FILE"
  verify_output=$(
    ENV_FILE="$ENV_FILE" "$REPO_ROOT/contracts/deploy.sh" --verify --chain "$chain" 2>&1
  )
  printf '%s\n' "$verify_output" | grep -q "Warning: verification is currently unsupported for $chain; deploying without verification"
  if grep -q 'forge verify-contract' "$LOG_FILE"; then
    echo "$chain --verify should not run forge verify-contract" >&2
    exit 1
  fi
done
