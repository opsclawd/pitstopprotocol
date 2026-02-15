# SPEC_INSTRUCTIONS/INDEX.md
Version: v1.0.0
Status: DRAFT (inventory locked; per-instruction status varies)

Authoritative instruction inventory for MVP (count: 12).

| # | Instruction | Status | Touches Tokens | Emits Events | Changes Market Status | Tests Required |
|---|-------------|--------|----------------|--------------|-----------------------|----------------|
| 1 | initialize | LOCKED | N | Y | N | unit + integration |
| 2 | create_market | Draft | Y | Y | Seeding init | integration + adversarial |
| 3 | add_outcome | Draft | N | Y | N | integration |
| 4 | finalize_seeding | Draft | N | Y | Seeding->Open | integration |
| 5 | place_bet | Draft | Y | Y | N | integration + invariant + adversarial |
| 6 | lock_market | Draft | N | Y | Open->Locked | integration |
| 7 | resolve_market | Draft | N | Y | Locked->Resolved | integration + adversarial |
| 8 | void_market | Draft | N | Y | Locked->Voided | integration |
| 9 | claim_resolved | Draft | Y | Y | N | integration + invariant |
|10 | claim_voided | Draft | Y | Y | N | integration + invariant |
|11 | sweep_remaining | Draft | Y | Y | Resolved/Voided->Swept (terminal accounting) | integration + adversarial |
|12 | cancel_market | Draft | Y | Y | Seeding->Voided | integration + adversarial |

## Rule
- Any new instruction file under `programs/**/instructions/*.rs` must have a matching spec file here.
- Instruction count is locked to 12 for MVP unless protocol version is bumped.
