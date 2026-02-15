# create_market Instruction (Issue #2)

## Purpose
Initializes a new market PDA for a race.

## Signature
- `race_id_hash: [u8; 32]`
- `open_ts: i64`
- `close_ts: i64`
- `driver_count: u8`
- `fee_bps: u16`

## Validation
- `driver_count >= 2 && driver_count <= 20`
- `close_ts > open_ts`
- `fee_bps <= 10_000`

## Initialization
- `status = Open`
- `winner_index = WINNER_UNSET (255)`
- all pool totals set to `0`
- `bump` captured from PDA derivation

## PDA
- Market PDA seeds: `b"market" + race_id_hash`
