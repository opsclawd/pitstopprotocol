# SPEC_PROTOCOL.md
Version: v1.0.0
Status: LOCKED (requires explicit version bump to change)

## Purpose
Single source of truth for PitStop protocol behavior.

## Lifecycle
- Config lifecycle: `Uninitialized -> Active (paused/unpaused)`
- Market lifecycle: `Seeding -> Open -> Locked -> (Resolved | Voided) -> Swept`

### Allowed transitions
- `initialize` creates Config
- `create_market` creates Market in Seeding
- `add_outcome` only during Seeding
- `finalize_seeding` transitions Seeding -> Open
- `lock_market` transitions Open -> Locked at/after lock timestamp
- `resolve_market` transitions Locked -> Resolved (oracle only)
- `void_market` transitions Locked -> Voided (oracle only)
- `sweep_remaining` only after claim window for Resolved/Voided
- `cancel_market` only in Seeding with zero pool + empty vault

## Trust model
- Authority/operator is trusted for market creation and operations.
- Oracle is trusted for resolution payload and winning outcome.
- Users rely on on-chain custody and deterministic payout math.

## Token custody
- USDC (6 decimals), SPL Token v1 only.
- Market vault is ATA owned by market PDA.
- Funds outflow only via claim instructions and sweep.

## Economic model
- fee = total_pool * fee_bps / 10_000
- prize_pool = total_pool - fee
- winner payout = position_amount * prize_pool / winner_pool (floor)
- dust remains in vault until sweep.

## Locked PDA derivations
- config: `["config"]`
- market: `["market", market_id]`
- outcome_pool: `["outcome", market_pda, outcome_id]`
- position: `["position", market_pda, user_pubkey, outcome_id]`

## Change control
Any protocol change must:
1) bump version in this file
2) update invariants/canonical specs
3) include migration notes and test impact
