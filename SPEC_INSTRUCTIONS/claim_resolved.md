# claim_resolved
Version: v1.0.2
Status: LOCKED

## Purpose
Allow users to claim winnings (or mark loser claim as zero payout) after resolution within claim window.

## Inputs
- `outcome_id: u8`

## Accounts
- user signer
- config
- market
- position mut PDA ["position", market, user, outcome_id]
- outcome_pool (for winner pool)
- user_usdc mut
- vault mut
- token_program pinned

## Preconditions
- Missing position PDA account -> framework account resolution failure (expected)

- market.status == Resolved -> `MarketNotResolved` (Swept also fails here)
- !position.claimed -> `AlreadyClaimed`
- outcome pool must exist for `(market, outcome_id)` and match seeds/fields -> `OutcomeMismatch`
- now <= resolution_timestamp + claim_window_secs -> `ClaimWindowExpired`

## Effects
- compute fee/prize/payout (floor math)
- if winner: transfer payout vault -> user_usdc
- if loser: payout = 0, no transfer
- mark position.claimed=true; store position.payout

## Events
- `Claimed`

## Postconditions
- no double claim
- vault decreases only by payout amounts

## Required tests
- CLR-HP-001..003, CLR-REJ-001..004, CLR-INV-001..002
- CLR-ORD-001: post-sweep claim fails by status error (`MarketNotResolved`) before any vault/account access error


## Security notes
- Claim path relies on status gating; zero vault balance is not an authorization mechanism.


## Failure ordering (locked)
- Status gate is evaluated before vault account usage.
- In Swept state, claim must fail with `MarketNotResolved` deterministically (not account-missing failure).
