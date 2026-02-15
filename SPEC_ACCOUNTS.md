# SPEC_ACCOUNTS.md
Version: v1.0.2
Status: LOCKED

Canonical account layout contract.

## Config
- authority: Pubkey
- oracle: Pubkey
- usdc_mint: Pubkey
- treasury: Pubkey
- treasury_authority: Pubkey
- fee_bps: u16
- paused: bool
- max_total_pool_per_market: u64
- max_bet_per_user_per_market: u64
- claim_window_secs: i64
- token_program: Pubkey

## Market
- market_id: [u8;32]
- event_id: [u8;32]
- lock_timestamp: i64
- outcome_count: u8
- max_outcomes: u8
- total_pool: u64 (gross historical pool; claims do not decrement)
- status: enum
- resolved_outcome: Option<u8>
- resolution_payload_hash: [u8;32]
- resolution_timestamp: i64 (0 pre-resolution)
- vault: Pubkey
- market_type: enum
- rules_version: u16

## OutcomePool
- market: Pubkey
- outcome_id: u8
- pool_amount: u64

## Position
- market: Pubkey
- user: Pubkey
- outcome_id: u8
- amount: u64
- claimed: bool
- payout: u64

## Rent/closure policy
- Vault ATA may be closed in cancel flow if empty.
- Vault ATA is closed in sweep flow after transferring remaining balance to treasury.
- OutcomePool reclaim optional (MVP may leave rent dust).
- Market account remains as historical record.
