# resolve_market
Version: v1.0.0
Status: LOCKED

## Purpose
Resolve a locked market by setting winning outcome and payload hash.

## Inputs
- `winning_outcome_id: u8`
- `payload_hash: [u8;32]`

## Accounts
- oracle signer
- config (oracle check)
- market mut
- winning_outcome_pool (validated to exist)

## Preconditions
- oracle == config.oracle -> `UnauthorizedOracle`
- market.status == Locked -> `MarketNotLocked`
- winning outcome must exist in market -> `InvalidOutcomeId`/`OutcomeMismatch`

## Effects
- market.status = Resolved
- market.resolved_outcome = Some(winning)
- market.resolution_payload_hash = payload_hash
- market.resolution_timestamp = now

## Events
- `MarketResolved`

## Required tests
- RSM-HP-001, RSM-REJ-001..004, RSM-ADV-001
