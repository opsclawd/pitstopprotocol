# SPEC_THREAT_MODEL.md
Version: v1.0.0

## Assumed adversaries
- Malicious clients submitting forged/invalid accounts or token programs
- Users attempting double claims or lifecycle bypass
- Unauthorized signer attempts for admin/oracle instructions

## Accepted trust assumptions (MVP)
- Authority and oracle are trusted entities.

## Defenses
- PDA seed constraints + account ownership checks
- token program pinning (SPL Token v1)
- status/time gating on all lifecycle transitions
- explicit caps + checked math
- claim gates (`claimed` boolean) + post-window sweep rules

## Out of scope
- decentralized oracle trust minimization
- censorship resistance of operator actions
