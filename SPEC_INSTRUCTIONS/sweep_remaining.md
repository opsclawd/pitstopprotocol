# sweep_remaining
Version: v1.0.0
Status: LOCKED

## Purpose
After claim window expires, transfer remaining vault balance to treasury.

## Inputs
- none

## Accounts
- authority signer
- config (authority check)
- market
- vault mut
- treasury mut (must equal config.treasury)
- token_program pinned

## Preconditions
- authority == config.authority -> `Unauthorized`
- market.status in {Resolved, Voided} -> `MarketNotResolved`/`MarketNotVoided`
- now > resolution_timestamp + claim_window_secs -> `ClaimWindowNotExpired`
- treasury constraints valid (mint+owner) -> `InvalidTreasuryMint`/`InvalidTreasuryOwner`

## Effects
- transfer full vault.amount -> treasury
- logical lifecycle terminal accounting state reached

## Events
- `Swept`

## Required tests
- SWP-HP-001, SWP-REJ-001..004, SWP-ADV-001
