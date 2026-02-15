# Spec: Issue #3 — `place_bet` (add-to-position only)

## Goal
Implement `place_bet` so users can place multiple bets in the same market **for the same driver** and aggregate stake in one Position PDA.

## Scope
- Add `place_bet` instruction
- Initialize Position PDA on first bet
- Aggregate `amount_lamports` on later bets
- Transfer SOL from bettor signer to Market PDA account
- Update `market.total_pool_lamports` and selected `driver_pools_lamports[index]`

## Non-goals
- Changing driver after first bet in a market
- Settlement/claim behavior
- SPL token support

## Accounts
- `bettor: Signer` (payer + source SOL)
- `market: Account<Market>` (destination SOL + state update)
- `position: Account<Position>` PDA
  - seeds: `b"position" + market_pubkey + bettor_pubkey`
  - init_if_needed, payer=bettor
- `system_program`

## Validation rules
1. `amount_lamports >= MIN_BET_LAMPORTS`
2. `market.status == Open`
3. `now < market.close_ts`
4. `driver_index < market.driver_count`
5. If `position.amount_lamports > 0`, then `position.driver_index == driver_index`
6. All additions use checked arithmetic (overflow-safe)

## State transitions
- First bet (new position):
  - set `user`, `market`, `driver_index`, `claimed=false`, `amount_lamports=0`, `bump`
- Every bet:
  - transfer lamports from bettor -> market account
  - increment `position.amount_lamports`
  - set `position.last_bet_ts = now`
  - increment `market.total_pool_lamports`
  - increment `market.driver_pools_lamports[driver_index]`

## Errors
- `MarketNotOpen`
- `BettingClosed`
- `InvalidDriverIndex`
- `DriverSelectionLocked`
- `BetTooSmall`
- `MathOverflow`

## Security considerations
- Add-to-position lock prevents cross-driver manipulation through a single position account.
- Transfer occurs before accounting updates; if transfer fails, tx aborts and state remains unchanged.
- Overflow checks protect pool/accounting consistency.

## Test plan (Issue #6)
- First bet initializes position and updates pools.
- Second bet on same driver aggregates correctly.
- Bet on different driver with existing position fails.
- Bet after close fails.
- Invalid index fails.
- Overflow paths return `MathOverflow`.

## As-built notes
- Added open window guard (`now >= open_ts`) in addition to close guard.
- Added defense-in-depth position invariant checks (`position.user`, `position.market`).
- Added market config sanity check (`driver_count <= MAX_DRIVERS`) during betting.
- Migration note: `init_if_needed` assumes fresh deployment for `Position` layout; if legacy smaller Position accounts exist, migration/realloc strategy is required.
