# PitStop Protocol

This repo is now in **spec-first + TDD pivot mode** for a USDC-based Anchor protocol.

## Current phase
- No new feature code until spec pack is approved.
- Source of truth is the spec set in root (currently DRAFT until checklist complete):
  - `SPEC_PROTOCOL.md`
  - `SPEC_INVARIANTS.md`
  - `SPEC_CANONICAL.md`
  - `SPEC_THREAT_MODEL.md`
  - `SPEC_ACCOUNTS.md`
  - `SPEC_STATE_MACHINE.md`
  - `SPEC_EVENTS.md`
  - `SPEC_ERRORS.md`
  - `SPEC_INSTRUCTIONS/INDEX.md`
  - `SPEC_INSTRUCTIONS/`

## Process gates
- 1 PR per issue
- Spec first -> tests first -> implementation
- Independent review pass -> fix pass -> final review-summary comment
- PR descriptions must include file-by-file summary

## Archived pre-pivot line
- tag: `archive-pre-usdc-pivot-2026-02-15`
- branch: `archive/pre-usdc-pivot-2026-02-15`


## Spec Pack Completion Checklist
- [ ] Instruction specs fully filled (no TODO sections)
- [ ] State schema spec added (fields/types/ranges)
- [ ] Error taxonomy spec added (error -> failure mode mapping)
- [ ] Canonical encoding includes byte-level format + test vectors
- [ ] Runnable test harness exists and passes smoke tests
- [ ] CI runs tests (not placeholders)


## Repository cleanup note
- Legacy pre-pivot app scaffolds and docs were removed.
- `packages/core/src/protocol_primitives.cjs` is retained intentionally as the canonical shared primitives module used by unit tests and future implementation code.
- `programs/pitstop/` is now scaffolded as the spec-aligned Anchor target directory for upcoming protocol implementation issues.

## Devnet deploy pipeline
Issue #106 adds deploy/runbook scaffolding for devnet:
- Runbook: `docs/devnet-deploy-runbook.md`
- Scripts: `scripts/devnet/`
- Workflow: `.github/workflows/devnet-deploy.yml`
