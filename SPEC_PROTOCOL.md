# SPEC_PROTOCOL.md
Version: v1.0.4
Status: LOCKED

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



## Protocol constants (authoritative)
Machine-readable source: `specs/constants.json`
- `USDC_DECIMALS = 6`
- `MAX_CLAIM_WINDOW_SECS = 7_776_000` (90 days)
- `REQUIRED_TOKEN_PROGRAM = Tokenkeg...` (SPL Token v1)

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


## Sweep semantics (locked)
- `Swept` is an explicit on-chain market status set by `sweep_remaining`.
- `sweep_remaining` is single-shot in normal lifecycle; repeat calls fail because market is Swept and vault is closed.


## Vault policy after sweep (locked)
- Sweep closes vault ATA after transferring full balance to treasury (rent reclaimed to configured close destination).
- Post-sweep, vault account no longer exists.
- Claims are rejected by status gate (market must be Resolved/Voided) and by missing-vault account constraints.
