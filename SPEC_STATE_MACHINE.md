# SPEC_STATE_MACHINE.md
Version: v1.0.4
Status: LOCKED

## Market States
- Seeding
- Open
- Locked
- Resolved
- Voided
- Swept (explicit on-chain terminal state after sweep action)

## Allowed transitions
- create_market => Seeding
- finalize_seeding: Seeding -> Open
- lock_market: Open -> Locked
- resolve_market: Locked -> Resolved
- void_market: Locked -> Voided
- cancel_market: Seeding -> Voided
- sweep_remaining: Resolved|Voided -> Swept (explicit on-chain terminal transition)

## Forbidden transitions (explicit)
- Seeding -> Locked (must open first)
- Open -> Resolved/Voided (must lock first)
- Locked -> Open
- Resolved -> Locked/Open/Seeding
- Voided -> Locked/Open/Seeding
- Swept -> any other state


## Swept terminal guarantees
- Swept is terminal and non-reversible.
- No mutating instruction is valid in Swept (claims, sweep, lock, resolve, void, cancel, add outcomes, finalize, place bet).
- Read-only queries remain allowed.


## Swept storage semantics
- Swept implies vault has been transferred and closed.
