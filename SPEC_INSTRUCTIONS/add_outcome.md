# add_outcome
Version: v1.0.0
Status: LOCKED

## Purpose
Add one outcome pool to a seeding market.

## Inputs
- `outcome_id: u8` (0..=99)

## Accounts
- authority signer
- config (authority check)
- market mut
- outcome_pool init PDA ["outcome", market, outcome_id]
- system_program

## Preconditions
- authority == config.authority -> `Unauthorized`
- market.status == Seeding -> `MarketNotSeeding`
- outcome_id <= 99 -> `InvalidOutcomeId`
- market.outcome_count < market.max_outcomes -> `MaxOutcomesReached`

## Effects
- create outcome_pool with pool_amount=0
- increment market.outcome_count by 1

## Token effects
- none

## Events
- `OutcomeAdded`

## Postconditions
- outcome exists and is bound to market
- outcome_count incremented exactly once

## Failure modes
- duplicate PDA create fails (system create/account in use)

## Security notes
- PDA uniqueness prevents duplicate outcome IDs.

## Required tests
- ADO-HP-001, ADO-REJ-001..005
