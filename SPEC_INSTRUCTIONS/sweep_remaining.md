# sweep_remaining
Version: v1.0.3
Status: LOCKED

## Purpose
After claim window expires, transfer remaining vault balance to treasury and close vault ATA.

## Inputs
- none

## Accounts
- authority signer
- config (authority check)
- market
- vault mut
- treasury mut (must equal config.treasury)
- token_program pinned
- close_destination: SystemAccount (rent recipient on vault close; expected = authority or treasury authority)

## Preconditions
- authority == config.authority -> `Unauthorized`
- market.status in {Resolved, Voided} -> `MarketNotResolved` (single deterministic error when not eligible, incl Swept)
- now > resolution_timestamp + claim_window_secs -> `ClaimWindowNotExpired`
- treasury constraints valid (mint+owner) -> `InvalidTreasuryMint`/`InvalidTreasuryOwner`

## Effects
- transfer full vault.amount -> treasury
- close vault ATA using market PDA signer seeds
- market.status = Swept (explicit on-chain terminal status)

## Events
- `Swept`

## Required tests
- SWP-HP-001, SWP-REJ-001..004, SWP-ADV-001


## Idempotency
- Second sweep call must fail because status is no longer Resolved/Voided (returns `MarketNotResolved`).


## Postconditions
- `market.status == Swept`
- vault account no longer exists
- subsequent sweep attempts fail lifecycle gate/account constraints


## Vault handling
- Vault is closed in sweep path to prevent grief-deposits after terminal settlement.
