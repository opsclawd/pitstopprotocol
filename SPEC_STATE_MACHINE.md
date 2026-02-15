# SPEC_STATE_MACHINE.md
Version: v1.0.0
Status: DRAFT

## Market States
- Seeding
- Open
- Locked
- Resolved
- Voided
- Swept (accounting terminal state after sweep action)

## Allowed transitions
- create_market => Seeding
- finalize_seeding: Seeding -> Open
- lock_market: Open -> Locked
- resolve_market: Locked -> Resolved
- void_market: Locked -> Voided
- cancel_market: Seeding -> Voided
- sweep_remaining: Resolved|Voided -> Swept (logical terminal for payout window lifecycle)

## Forbidden transitions (explicit)
- Seeding -> Locked (must open first)
- Open -> Resolved/Voided (must lock first)
- Locked -> Open
- Resolved -> Locked/Open/Seeding
- Voided -> Locked/Open/Seeding
- Swept -> any other state
