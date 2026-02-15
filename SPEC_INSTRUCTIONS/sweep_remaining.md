# sweep_remaining
Version: v1.0.2
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
- market.status in {Resolved, Voided} -> `MarketNotResolved` (single deterministic error when not eligible, incl Swept)
- now > resolution_timestamp + claim_window_secs -> `ClaimWindowNotExpired`
- treasury constraints valid (mint+owner) -> `InvalidTreasuryMint`/`InvalidTreasuryOwner`

## Effects
- transfer full vault.amount -> treasury
- market.status = Swept (explicit on-chain terminal status)

## Events
- `Swept`

## Required tests
- SWP-HP-001, SWP-REJ-001..004, SWP-ADV-001


## Idempotency
- Second sweep call must fail because status is no longer Resolved/Voided (returns `MarketNotResolved`).


## Postconditions
- `market.status == Swept`
- `vault.amount == 0`
- subsequent sweep attempts fail lifecycle gate


## Vault handling
- Vault is NOT closed in sweep path (remains open with zero balance).
