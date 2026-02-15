# Program Account Model (Issue #1)

MVP constraints:
- SOL-only escrow
- Winner-only market
- Add-to-position betting (user can place multiple bets on same driver in same market)
- No floats, no `Vec<String>` on-chain

## Seeds

### Market PDA
- Seeds: `b"market"`, `race_id_hash[32]`
- One market per race.

### Position PDA (user bet position)
- Seeds: `b"position"`, `market_pubkey`, `user_pubkey`
- Exactly one position account per `(market, user)`.
- Additive betting only on same `driver_index`.

## Constants
- `MAX_DRIVERS = 20`
- `FEE_BPS = 500` (5%, configurable per market)
- `BPS_DENOMINATOR = 10_000`
- `MIN_BET_LAMPORTS = 1_000_000` (0.001 SOL, tuning value)

## Account shapes

### Market
- `authority: Pubkey`
- `race_id_hash: [u8; 32]`
- `open_ts: i64`
- `close_ts: i64`
- `status: u8` (`Open=0, Settled=1, Cancelled=2`)
- `winner_index: u8` (`255` = unset)
- `driver_count: u8` (must be `<= 20`)
- `fee_bps: u16`
- `total_pool_lamports: u64`
- `driver_pools_lamports: [u64; 20]`
- `winner_pool_lamports: u64`
- `bump: u8`

### Position
- `user: Pubkey`
- `market: Pubkey`
- `driver_index: u8`
- `amount_lamports: u64`
- `claimed: bool`
- `last_bet_ts: i64`
- `bump: u8`

## Rules
- `create_market`
  - `driver_count in 2..=20`
  - `close_ts > open_ts`
- `place_bet`
  - market status must be open
  - now `< close_ts`
  - amount `>= MIN_BET_LAMPORTS`
  - if position exists: must use same `driver_index`, then increment `amount_lamports`
- `settle_market`
  - after `close_ts`
  - one-time operation
  - set `winner_index` and `winner_pool_lamports`
- `claim`
  - requires settled market
  - one-time per position
  - payout math uses integer arithmetic only

## Notes
- Driver metadata (names, images, stats) stays off-chain in app/API.
- Real-time odds come from market account subscription and client-side calculation.
