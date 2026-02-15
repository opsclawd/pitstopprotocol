# PitStop Protocol

This repo is now in **spec-first + TDD pivot mode** for a USDC-based Anchor protocol.

## Current phase
- No new feature code until spec pack is approved.
- Source of truth is the spec set in root (currently DRAFT until checklist complete):
  - `SPEC_PROTOCOL.md`
  - `SPEC_INVARIANTS.md`
  - `SPEC_CANONICAL.md`
  - `SPEC_THREAT_MODEL.md`
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
