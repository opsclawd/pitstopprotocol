# Spec: Issue #4 — `settle_market`

## Goal
Implement `settle_market` to finalize a market after betting closes by setting winner and locking market state.

## Scope
- Add `settle_market` instruction
- Authority-only settlement
- Enforce settlement timing (`now >= close_ts`)
- Set `winner_index`
- Set `winner_pool_lamports`
- Transition status to Settled

## Non-goals
- Claim/refund payout execution (Issue #5)
- Oracle fetching logic (Issue #14)

## Accounts
- `authority: Signer` (must match `market.authority`)
- `market: Account<Market>` PDA
  - seeds: `b"market" + market.authority + market.race_id_hash`

## Validation rules
1. `market.status == Open`
2. `authority.key == market.authority`
3. `now >= market.close_ts`
4. `winner_index < market.driver_count`

## State transitions
- `market.winner_index = winner_index`
- `market.winner_pool_lamports = market.driver_pools_lamports[winner_index]`
- `market.status = Settled`

## winner_pool == 0 behavior
If winner pool is zero, settlement still succeeds. Claim logic (Issue #5) will treat this as refund path trigger.

## Errors
- `Unauthorized`
- `MarketNotOpen`
- `SettlementTooEarly`
- `InvalidWinnerIndex`

## Test plan (Issue #6)
- Happy path settle after close.
- Unauthorized authority fails.
- Settle before close fails.
- Invalid winner index fails.
- Settled market cannot be settled twice.
- `winner_pool_lamports` snapshot correct for both non-zero and zero winner pools.

## As-built notes
- Implemented in `programs/pitstop_protocol/src/instructions/settle_market.rs`.
- Added Anchor `has_one = authority` constraint for clearer authorization semantics.
- Added defense-in-depth `driver_count <= MAX_DRIVERS` sanity check before array indexing.
