#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
PROGRAM_ID="$(node -e "const x=require('$ROOT/configs/program-ids.json');console.log(x.pitstop.devnet)")"

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

echo "Checking deployed program account..."
solana program show "$PROGRAM_ID" --url devnet >/dev/null

echo "Checking IDL fetch path..."
anchor idl fetch -o /tmp/pitstop.idl.json "$PROGRAM_ID" --provider.cluster devnet >/dev/null
node -e "const f=require('fs');const idl=JSON.parse(f.readFileSync('/tmp/pitstop.idl.json','utf8'));const must=['initialize','create_market','add_outcome','finalize_seeding'];const names=new Set((idl.instructions||[]).map(i=>i.name));const miss=must.filter(x=>!names.has(x));if(miss.length){console.error('IDL missing instructions: '+miss.join(','));process.exit(1)};console.log('IDL instruction smoke ok');"

echo "devnet smoke checks ok"
