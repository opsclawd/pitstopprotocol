#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
IDL_SRC="$ROOT/target/idl/pitstop.json"
OUT_DIR="$ROOT/artifacts/idl"

if [[ ! -f "$IDL_SRC" ]]; then
  echo "Missing IDL at $IDL_SRC (run anchor build first)" >&2
  exit 1
fi

mkdir -p "$OUT_DIR"
STAMP="$(date -u +%Y%m%dT%H%M%SZ)"
cp "$IDL_SRC" "$OUT_DIR/pitstop.$STAMP.json"
cp "$IDL_SRC" "$OUT_DIR/pitstop.latest.json"

echo "Published IDL:"
echo "- $OUT_DIR/pitstop.$STAMP.json"
echo "- $OUT_DIR/pitstop.latest.json"
