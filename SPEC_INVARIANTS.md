# SPEC_INVARIANTS.md
Version: v1.0.0

## Always-true invariants (post successful tx)
1. `sum(outcome_pool.pool_amount) == market.total_pool`
2. Pre-resolution: `vault.amount == market.total_pool`
3. No double claim: once `position.claimed == true`, later claim must fail.
4. Vault outflow only through `claim_*` and `sweep_remaining`.

## Stage invariants
### Resolved
- `resolved_outcome` is set and corresponds to existing outcome.
- `resolution_timestamp > 0`.

### Voided
- claims refund principal exactly.

### Swept
- `vault.amount == 0`.

## Payout conservation
- sum(winner payouts) <= prize_pool
- dust = prize_pool - sum(winner payouts)
- dust bounded by floor-division behavior and winner position count.
