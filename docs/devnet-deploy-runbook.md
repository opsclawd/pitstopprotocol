# Devnet Deploy Runbook (Issue #106)

## Prereqs
- Anchor CLI `0.30.1`
- Solana CLI
- funded devnet deploy keypair at `~/.config/solana/id.json`
- `configs/program-ids.json` has non-TODO `pitstop.devnet`

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
Checks:
- program account exists on devnet
- IDL can be fetched
- required instruction names exist in fetched IDL

## GitHub Actions
Workflow: `.github/workflows/devnet-deploy.yml`
- Trigger manually with `workflow_dispatch`
- Secret required: `SOLANA_DEVNET_KEYPAIR_B64` (base64 JSON keypair)
- Optional input: `run_smoke_only=true`
