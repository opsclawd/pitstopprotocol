#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
cd "$ROOT"

if ! command -v anchor >/dev/null 2>&1; then
  echo "anchor CLI not found" >&2
  exit 1
fi
if ! command -v solana >/dev/null 2>&1; then
  echo "solana CLI not found" >&2
  exit 1
fi

# Optional CI path: inject keypair via DEVNET_WALLET_JSON_B64.
if [[ -n "${DEVNET_WALLET_JSON_B64:-}" ]]; then
  mkdir -p "$HOME/.config/solana"
  echo "$DEVNET_WALLET_JSON_B64" | base64 -d > "$HOME/.config/solana/id.json"
  chmod 600 "$HOME/.config/solana/id.json"
fi

node scripts/devnet/apply_program_ids.js

anchor build
anchor deploy --provider.cluster devnet

scripts/devnet/publish_idl.sh
scripts/devnet/smoke.sh

echo "Devnet deploy pipeline complete."
