# lock_market
Version: v1.0.0
Status: LOCKED

## Purpose
Stop betting by transitioning an open market to locked at/after lock timestamp.

## Inputs
- none

## Accounts
- authority signer
- config (authority check)
- market mut

## Preconditions
- authority == config.authority -> `Unauthorized`
- market.status == Open -> `MarketNotOpen`
- now >= lock_timestamp -> `TooEarlyToLock`

## Effects
- market.status = Locked

## Events
- `MarketLocked`

## Required tests
- LKM-HP-001, LKM-REJ-001..003
