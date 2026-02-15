# void_market
Version: v1.0.0
Status: LOCKED

## Purpose
Void a locked market so users can claim refunds.

## Inputs
- `payload_hash: [u8;32]`

## Accounts
- oracle signer
- config (oracle check)
- market mut

## Preconditions
- oracle == config.oracle -> `UnauthorizedOracle`
- market.status == Locked -> `MarketNotLocked`

## Effects
- market.status = Voided
- market.resolved_outcome = None
- market.resolution_payload_hash set
- market.resolution_timestamp = now

## Events
- `MarketVoided`

## Required tests
- VDM-HP-001, VDM-REJ-001..003


## Event contract link
- Event spec reference: `SPEC_EVENTS.md` -> `MarketVoided`.
