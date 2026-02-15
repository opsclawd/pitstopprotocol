# cancel_market
Version: v1.0.1
Status: LOCKED

## Purpose
Recovery path to void dead markets during Seeding before bets exist.

## Inputs
- none

## Accounts
- authority signer mut
- config (authority check)
- market mut
- vault mut
- token_program pinned
- close_destination: SystemAccount (rent recipient on vault close; expected = authority)

## Preconditions
- authority == config.authority -> `Unauthorized`
- market.status == Seeding -> `MarketNotSeeding`
- now >= lock_timestamp -> `TooLateToCancel`
- market.total_pool == 0 -> `MarketHasBets`
- vault.amount == 0 -> `VaultNotEmpty`

## Effects
- close vault ATA with market PDA signer seeds to `close_destination`
- market.status = Voided
- set resolution timestamp/hash baseline

## Events
- `MarketCancelled` (and optional `MarketVoided`)

## Required tests
- CNL-HP-001, CNL-REJ-001..005, CNL-ADV-001


## Event contract link
- Event spec reference: `SPEC_EVENTS.md` -> `MarketCancelled`.
