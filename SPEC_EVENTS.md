# SPEC_EVENTS.md
Version: v1.2.0
Status: LOCKED

Event contract for indexing/API surfaces.

## Canonical event list (locked)
- ConfigInitialized { authority, oracle, usdc_mint, treasury, fee_bps, timestamp }
- MarketCreated { market, market_id, event_id, lock_timestamp, max_outcomes, market_type, rules_version, timestamp }
- OutcomeAdded { market, outcome_id, outcome_count, timestamp }
- MarketOpened { market, timestamp }
- BetPlaced { market, user, outcome_id, amount, market_total_pool, outcome_pool_amount, timestamp }
- MarketLocked { market, timestamp }
- MarketResolved { market, winning_outcome, payload_hash, resolution_timestamp }
- MarketVoided { market, payload_hash, resolution_timestamp }
- Claimed { market, user, outcome_id, payout, claimed_at }
- MarketSweptEvent { market, amount, to_treasury, timestamp }
- MarketCancelled { market, timestamp }

## Emission rules (must-emit matrix)

| Instruction | Must emit | Event | Notes |
|---|---|---|---|
| initialize | Yes | ConfigInitialized | exactly once on successful config init |
| create_market | Yes | MarketCreated | emitted after market+vault init success |
| add_outcome | Yes | OutcomeAdded | includes updated outcome_count |
| finalize_seeding | Yes | MarketOpened | on Seeding->Open transition |
| place_bet | Yes | BetPlaced | emitted after transfer + state updates |
| lock_market | Yes | MarketLocked | on Open->Locked transition |
| resolve_market | Yes | MarketResolved | on Locked->Resolved transition |
| void_market | Yes | MarketVoided | on Locked->Voided transition |
| claim_resolved | Yes | Claimed | payout may be 0 for losers |
| claim_voided | Yes | Claimed | payout equals refunded principal |
| sweep_remaining | Yes | MarketSweptEvent | emitted on successful sweep transfer |
| cancel_market | Yes | MarketCancelled | emitted on successful cancel path |

## Determinism requirements
- All amount fields are in base token units (USDC 6 decimals).
- Event timestamp fields must use on-chain clock (`Clock::get()?.unix_timestamp`).
- Lifecycle events must align exactly with state transitions in `SPEC_STATE_MACHINE.md`.

## Error/event interaction
- No event must be emitted on failed instructions.
- Event emission happens only after all state/token effects for the instruction succeed.

## Test requirements
- EVT-MTX-001: every successful instruction emits exactly its required event.
- EVT-MTX-002: failed instruction emits no event.
- EVT-MTX-003: event payload fields match post-state values.
- EVT-MTX-004: lifecycle events match allowed transitions only.
