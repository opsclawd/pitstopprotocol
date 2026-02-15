# SPEC_EVENTS.md
Version: v1.0.1
Status: DRAFT

Event contract for indexing/API surfaces.

## Events (minimum)
- ConfigInitialized { authority, oracle, usdc_mint, treasury, fee_bps, timestamp }
- MarketCreated { market, market_id, event_id, lock_timestamp, max_outcomes, timestamp }
- OutcomeAdded { market, outcome_id, timestamp }
- MarketOpened { market, timestamp }
- MarketLocked { market, timestamp }
- MarketResolved { market, winning_outcome, payload_hash, timestamp }
- MarketVoided { market, payload_hash, timestamp }
- BetPlaced { market, user, outcome_id, amount, timestamp }
- Claimed { market, user, outcome_id, payout, timestamp }
- MarketSweptEvent { market, amount, to_treasury, timestamp }
- MarketCancelled { market, timestamp }

## Must-emit rules
- Lifecycle transitions must emit corresponding lifecycle event.
- Token-moving instructions must emit amount-bearing events.
- `sweep_remaining` must emit `MarketSweptEvent` exactly once in normal lifecycle flow (single-shot).
