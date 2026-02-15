# sweep_remaining
Version: v1.0.4
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
- SWP-AUTH-001: non-authority sweep rejected (`Unauthorized`)
- SWP-WIN-001: claim window not expired rejected (`ClaimWindowNotExpired`)
- SWP-SEED-001: vault close uses market PDA signer seeds and closes vault account
- SWP-IDEM-001: repeat sweep fails deterministically via status gate (`MarketNotResolved`)


## Idempotency
- Second sweep call must fail because status is no longer Resolved/Voided (returns `MarketNotResolved`).


## Postconditions
- `market.status == Swept`
- vault account no longer exists
- subsequent sweep attempts fail lifecycle gate/account constraints


## Vault handling
- Vault is closed in sweep path to prevent grief-deposits after terminal settlement.


## Authorization and gating (locked)
- Requires `authority == config.authority` (MVP, not permissionless).
- Requires `market.status in {Resolved, Voided}` and `now > resolution_timestamp + claim_window_secs`.


## Close semantics (locked)
- Vault close must use market PDA signer seeds.
- Tests must assert: vault account is closed (account fetch fails) and treasury balance increases by swept amount.


## Idempotency (locked)
- Re-running sweep must fail deterministically by status gate (`MarketNotResolved`), not by missing-account error.


## Event contract link
- Event spec reference: `SPEC_EVENTS.md` -> `MarketSweptEvent`.
