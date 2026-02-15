# place_bet
Version: v1.0.5
Status: LOCKED

## Purpose
Transfer USDC from user to market vault and record/increment user position + pool totals.

## Inputs
- `outcome_id: u8`
- `amount: u64` (USDC base units, >0)

## Accounts
- config
- market mut
- outcome_pool mut PDA ["outcome", market, outcome_id]
- position init_if_needed PDA ["position", market, user, outcome_id]
- user signer
- user_usdc token account (owner=user, mint=config.usdc_mint)
- vault token account (key==market.vault, mint=config.usdc_mint)
- token_program (pinned)
- system_program

## Preconditions
- !config.paused -> `ProtocolPaused`
- market.status == Open -> `MarketNotOpen`
- now >= market.lock_timestamp -> `BettingClosed`
- outcome_id <= 99 -> `InvalidOutcomeId`
- market.outcome_count != market.max_outcomes -> `MarketNotReady`
- amount > 0 -> `ZeroAmount`
- caps not exceeded -> `MarketCapExceeded` / `UserBetCapExceeded`
- outcome_id must reference an initialized OutcomePool PDA for this market
  - wrong PDA relation -> `OutcomeMismatch`
  - missing/uninitialized PDA -> framework account failure unless wrapped
- outcome pool market/outcome match -> `OutcomeMismatch (covers both: wrong PDA passed, and PDA not initialized/missing)`

## Effects
- token transfer user_usdc -> vault by `amount`
- outcome_pool.pool_amount += amount
- market.total_pool += amount
- position init or increment amount

## Events
- `BetPlaced`

## Postconditions
- sum(outcome pools) == market.total_pool
- pre-resolution vault.amount == market.total_pool

## Required tests
- PBT-HP-001..002, PBT-REJ-001..010, PBT-INV-001..002, PBT-ADV-001..004


## Outcome existence test requirement
- Tests must include both cases: wrong PDA and missing/uninitialized PDA, each mapping to `OutcomeMismatch`.


## Event contract link
- Event spec reference: `SPEC_EVENTS.md` -> `BetPlaced`.
