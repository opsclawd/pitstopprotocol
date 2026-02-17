#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PROGRAM_ID="$(node -e "const x=require('$ROOT/configs/program-ids.json');console.log(x.pitstop.devnet)")"
IDL_SOURCE="${1:-auto}"

if [[ "$IDL_SOURCE" == "--source" ]]; then
  IDL_SOURCE="${2:-auto}"
fi

if [[ -z "$PROGRAM_ID" || "$PROGRAM_ID" == *"TODO"* ]]; then
  echo "Invalid devnet program id in configs/program-ids.json" >&2
  exit 1
fi

if ! command -v solana >/dev/null 2>&1; then
  echo "solana CLI not found" >&2
  exit 1
fi
if ! command -v anchor >/dev/null 2>&1; then
  echo "anchor CLI not found" >&2
  exit 1
fi

# Optional CI path: inject wallet via DEVNET_WALLET_JSON_B64.
if [[ -n "${DEVNET_WALLET_JSON_B64:-}" ]]; then
  mkdir -p "$HOME/.config/solana"
  echo "$DEVNET_WALLET_JSON_B64" | base64 -d > "$HOME/.config/solana/id.json"
  chmod 600 "$HOME/.config/solana/id.json"
fi

echo "Checking deployed program account..."
solana program show "$PROGRAM_ID" --url devnet >/dev/null

LOCAL_IDL="$ROOT/artifacts/idl/pitstop.latest.json"
if [[ "$IDL_SOURCE" == "auto" ]]; then
  if [[ -f "$LOCAL_IDL" ]]; then
    IDL_SOURCE="local"
  else
    IDL_SOURCE="onchain"
  fi
fi

if [[ "$IDL_SOURCE" == "local" ]]; then
  if [[ ! -f "$LOCAL_IDL" ]]; then
    echo "Missing local IDL: $LOCAL_IDL" >&2
    exit 1
  fi
  IDL_JSON_PATH="$LOCAL_IDL"
  echo "Checking local IDL at $IDL_JSON_PATH..."
elif [[ "$IDL_SOURCE" == "onchain" ]]; then
  IDL_JSON_PATH="/tmp/pitstop.idl.json"
  echo "Checking on-chain IDL fetch path..."
  anchor idl fetch -o "$IDL_JSON_PATH" "$PROGRAM_ID" --provider.cluster devnet >/dev/null
else
  echo "Invalid IDL source: $IDL_SOURCE (expected: auto|local|onchain)" >&2
  exit 1
fi

node -e "const f=require('fs');const idl=JSON.parse(f.readFileSync(process.argv[1],'utf8'));const must=['initialize','create_market','add_outcome','finalize_seeding'];const names=new Set((idl.instructions||[]).map(i=>i.name));const miss=must.filter(x=>!names.has(x));if(miss.length){console.error('IDL missing instructions: '+miss.join(','));process.exit(1)};console.log('IDL instruction smoke ok');" "$IDL_JSON_PATH"

echo "devnet smoke checks ok"
