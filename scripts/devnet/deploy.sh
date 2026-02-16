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

# Optional CI path: inject deploy wallet via DEVNET_WALLET_JSON_B64.
if [[ -n "${DEVNET_WALLET_JSON_B64:-}" ]]; then
  mkdir -p "$HOME/.config/solana"
  echo "$DEVNET_WALLET_JSON_B64" | base64 -d > "$HOME/.config/solana/id.json"
  chmod 600 "$HOME/.config/solana/id.json"
fi

# Optional CI path: inject canonical program keypair for pitstop deploy id.
if [[ -n "${DEVNET_PROGRAM_KEYPAIR_B64:-}" ]]; then
  mkdir -p "$ROOT/target/deploy"
  echo "$DEVNET_PROGRAM_KEYPAIR_B64" | base64 -d > "$ROOT/target/deploy/pitstop-keypair.json"
  chmod 600 "$ROOT/target/deploy/pitstop-keypair.json"
fi

node scripts/devnet/apply_program_ids.js

PROGRAM_ID="$(node -e "const x=require('$ROOT/configs/program-ids.json');console.log(x.pitstop.devnet)")"
PROGRAM_KEYPAIR_PATH="$ROOT/target/deploy/pitstop-keypair.json"

if [[ ! -f "$PROGRAM_KEYPAIR_PATH" ]]; then
  echo "Missing program keypair: $PROGRAM_KEYPAIR_PATH" >&2
  echo "Provide DEVNET_PROGRAM_KEYPAIR_B64 or create target/deploy/pitstop-keypair.json" >&2
  exit 1
fi

KEYPAIR_PUBKEY="$(solana-keygen pubkey "$PROGRAM_KEYPAIR_PATH")"
if [[ "$KEYPAIR_PUBKEY" != "$PROGRAM_ID" ]]; then
  echo "Program keypair pubkey mismatch:" >&2
  echo "- configs/program-ids.json pitstop.devnet: $PROGRAM_ID" >&2
  echo "- target/deploy/pitstop-keypair.json: $KEYPAIR_PUBKEY" >&2
  exit 1
fi

anchor build
anchor deploy --provider.cluster devnet

scripts/devnet/publish_idl.sh
scripts/devnet/smoke.sh

echo "Devnet deploy pipeline complete."
