# Devnet Deploy Runbook (Issue #106)

## Prereqs
- Anchor CLI `0.30.1`
- Solana CLI
- funded devnet deploy wallet at `~/.config/solana/id.json`
- canonical program keypair at `target/deploy/pitstop-keypair.json`
- `configs/program-ids.json` has non-TODO `pitstop.devnet`
- program keypair pubkey must match `configs/program-ids.json pitstop.devnet`

## Program ID management
Source of truth:
- `configs/program-ids.json` (`pitstop.devnet`)

Apply to code/config:
```bash
node scripts/devnet/apply_program_ids.js
```

Check-only (CI-safe):
```bash
node scripts/devnet/apply_program_ids.js --check
```

## One-command local devnet deploy
```bash
scripts/devnet/deploy.sh
```

This performs:
1. apply program ids
2. `anchor build`
3. `anchor deploy --provider.cluster devnet`
4. publish IDL to `artifacts/idl/`
5. smoke checks (`scripts/devnet/smoke.sh`)

## IDL publishing
```bash
scripts/devnet/publish_idl.sh
```
Outputs:
- `artifacts/idl/pitstop.<timestamp>.json`
- `artifacts/idl/pitstop.latest.json`

## Smoke verification
```bash
scripts/devnet/smoke.sh
```
Default mode is `auto`:
- use local IDL at `artifacts/idl/pitstop.latest.json` when present
- otherwise fetch IDL from devnet on-chain metadata

Explicit modes:
```bash
scripts/devnet/smoke.sh --source local
scripts/devnet/smoke.sh --source onchain
```
Checks:
- program account exists on devnet
- IDL source is readable (local artifact or on-chain fetch)
- required instruction names exist in checked IDL

CI note:
- `smoke.sh` supports `DEVNET_WALLET_JSON_B64` and will materialize `~/.config/solana/id.json` when provided.

## GitHub Actions
Workflow: `.github/workflows/devnet-deploy.yml`
- Trigger manually with `workflow_dispatch`
- Secrets required for deploy:
  - `SOLANA_DEVNET_KEYPAIR_B64` (base64 JSON deploy wallet)
  - `SOLANA_DEVNET_PROGRAM_KEYPAIR_B64` (base64 JSON program keypair matching `pitstop.devnet`)
- Optional input: `run_smoke_only=true`
