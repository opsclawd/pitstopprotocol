# SPEC_CANONICAL.md
Version: v1.0.0

## Canonical event descriptor -> event_id
- Input descriptor fields: sport, season, round, session
- Deterministic encoding required (no locale/timezone ambiguity)
- `event_id = sha256(canonical_encoded_descriptor)`

## market_id
- `market_id = sha256(event_id || market_type_byte || rules_version_le_u16)`
- On-chain must verify provided market_id matches computed value.

## Resolution payload hash
- Payload hash is canonicalized bytes hash (`sha256`) of off-chain payload.
- Same semantic payload must produce same bytes before hashing.

## Timestamps
- Unix seconds only.
- Reject obvious milliseconds in clients/operators.
