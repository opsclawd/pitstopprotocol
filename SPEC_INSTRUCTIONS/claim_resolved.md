# claim_resolved
Version: v1.0.1
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

- market.status == Resolved -> `MarketNotResolved`
- !position.claimed -> `AlreadyClaimed`
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
