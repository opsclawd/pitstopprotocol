# SPEC_INSTRUCTIONS/INDEX.md
Version: v1.0.2
Status: DRAFT (inventory locked; per-instruction status varies)

Authoritative instruction inventory for MVP (count: 12).

| # | Instruction | Status | Touches Tokens | Emits Events | Changes Market Status | Tests Required |
|---|-------------|--------|----------------|--------------|-----------------------|----------------|
| 1 | initialize | LOCKED | N | Y | N | unit + integration |
| 2 | create_market | LOCKED | Y | Y | Seeding init | integration + adversarial |
| 3 | add_outcome | LOCKED | N | Y | N | integration |
| 4 | finalize_seeding | LOCKED | N | Y | Seeding->Open | integration |
| 5 | place_bet | LOCKED | Y | Y | N | integration + invariant + adversarial |
| 6 | lock_market | LOCKED | N | Y | Open->Locked | integration |
| 7 | resolve_market | LOCKED | N | Y | Locked->Resolved | integration + adversarial |
| 8 | void_market | LOCKED | N | Y | Locked->Voided | integration |
| 9 | claim_resolved | LOCKED | Y | Y | N | integration + invariant |
|10 | claim_voided | Draft | Y | Y | N | integration + invariant |
|11 | sweep_remaining | LOCKED | Y | Y | Resolved/Voided->Swept (terminal accounting) | integration + adversarial |
|12 | cancel_market | LOCKED | Y | Y | Seeding->Voided | integration + adversarial |

## Rule
- Any new instruction file under `programs/**/instructions/*.rs` must have a matching spec file here.
- Instruction count is locked to 12 for MVP unless protocol version is bumped.
