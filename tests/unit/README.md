# Unit tests (Issue #58)

- `canonical_ids.spec.js` — canonical descriptor/event_id and market_id vectors
- `timestamp_rules.spec.js` — seconds-only validation and bounds
- `math.spec.js` — fee/prize/payout math, floor behavior, dust sanity

These tests are pure and deterministic (no chain dependency).
