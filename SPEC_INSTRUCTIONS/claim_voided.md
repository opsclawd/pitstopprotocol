# claim_voided
Version: v1.0.3
Status: LOCKED

## Purpose
Refund full stake for positions in a voided market within claim window.

## Inputs
- `outcome_id: u8`

## Accounts
- user signer
- config
- market
- position mut
- user_usdc mut
- vault mut
- token_program pinned

## Preconditions
- Missing position PDA account -> framework account resolution failure (expected)

- market.status == Voided -> `MarketNotVoided` (Swept also fails here)
- !position.claimed -> `AlreadyClaimed`
- now <= resolution_timestamp + claim_window_secs -> `ClaimWindowExpired`

## Effects
- transfer payout=position.amount from vault -> user_usdc
- mark claimed and set payout

## Events
- `Claimed`

## Required tests
- CLV-HP-001, CLV-REJ-001..003, CLV-INV-001
- CLV-ORD-001: post-sweep claim fails by status error (`MarketNotVoided`) before any vault/account access error


## Security notes
- Claim path relies on status gating; zero vault balance is not an authorization mechanism.


## Failure ordering (locked)
- Status gate is evaluated before vault account usage.
- In Swept state, claim must fail with `MarketNotVoided` deterministically (not account-missing failure).


## Event contract link
- Event spec reference: `SPEC_EVENTS.md` -> `Claimed`.
