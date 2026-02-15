# SPEC_CANONICAL.md
Version: v1.0.1
Status: LOCKED

## Canonical descriptor encoding
- UTF-8 JSON
- keys sorted lexicographically
- no whitespace
- field order canonicalized by sorter (not caller input order)

Example descriptor object:
{"round":"05","season":"2026","session":"race","sport":"f1"}

event_id = sha256(canonical_json_bytes)

## market_id bytes
market_id = sha256(event_id(32) || market_type_byte(1) || rules_version_le_u16(2))
- rules_version endianness: little-endian

## Golden vectors
Vector A
- descriptor_json: {"round":"05","season":"2026","session":"race","sport":"f1"}
- canonical_bytes_hex: 7b22726f756e64223a223035222c22736561736f6e223a2232303236222c2273657373696f6e223a2272616365222c2273706f7274223a226631227d
- event_id_hex: 5621e7f82cd0b15b457944898ab557629067d4256eaa5b7dc6cec414d5c66a7f

Vector B
- event_id_hex: 0000000000000000000000000000000000000000000000000000000000000000
- market_type_byte: 00
- rules_version_le_u16: 0100
- market_id_hex: b17820b1fb10fa804a7147ca7fd1e1666c62ef002e9adfd12019b35a28377664
