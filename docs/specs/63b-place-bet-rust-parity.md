# 63b PlaceBet Rust Parity Notes

This document maps Rust parity implementation to locked PlaceBet spec semantics.

## Source files
- `programs/pitstop/src/instructions/place_bet.rs`
- `programs/pitstop/src/error.rs`
- `programs/pitstop/src/events.rs`
- `programs/pitstop/src/state.rs`

## Precondition mapping (PBT-REJ)
- PBT-REJ-001 -> `ProtocolPaused`
- PBT-REJ-002 -> `MarketNotOpen`
- PBT-REJ-003 -> `BettingClosed`
- PBT-REJ-004 -> `InvalidOutcomeId`
- PBT-REJ-005 -> `MarketNotReady`
- PBT-REJ-006 -> `ZeroAmount`
- PBT-REJ-007 -> `MarketCapExceeded`
- PBT-REJ-008 -> `UserBetCapExceeded`
- PBT-REJ-009 -> `OutcomeMismatch` (wrong relation + modeled missing/uninitialized)
- PBT-REJ-010 -> `InvalidTokenProgram`
- checked arithmetic overflow -> `Overflow`

## Effect/event mapping
On success:
- `market.total_pool += amount`
- `outcome_pool.pool_amount += amount`
- `position.amount += amount`
- `vault_amount += amount`
- emit `BetPlaced`

## Notes
- This is parity logic for spec conformance.
- Full Anchor on-chain account/CPI wiring follows in integration pass.
