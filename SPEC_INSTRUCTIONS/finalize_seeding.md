# finalize_seeding
Version: v1.0.1
Status: LOCKED

## Purpose
Transition market from Seeding to Open once all outcomes are seeded.

## Inputs
- none

## Accounts
- market mut
- authority signer
- config (authority check)

## Preconditions
- authority == config.authority -> `Unauthorized`
- market.status == Seeding -> `MarketNotSeeding`
- market.outcome_count != market.max_outcomes -> `SeedingIncomplete`
- now >= lock_timestamp -> `TooLateToOpen`

## Effects
- market.status = Open

## Token effects
- none

## Events
- `MarketOpened`

## Postconditions
- market may accept bets until lock timestamp.

## Required tests
- FSE-HP-001, FSE-REJ-001..004


## Event contract link
- Event spec reference: `SPEC_EVENTS.md` -> `MarketOpened`.
