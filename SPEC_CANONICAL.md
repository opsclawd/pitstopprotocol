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


## Encoding format (draft)
- Descriptor encoded as UTF-8 JSON with sorted keys and no extra whitespace.
- market_id input bytes: event_id(32) || market_type_byte(1) || rules_version_le_u16(2).

## Test vectors
1) descriptor={"sport":"f1","season":"2026","round":"05","session":"race"}
   - expected_event_id: TBD
2) event_id=<vector1>, market_type=0, rules_version=1
   - expected_market_id: TBD
