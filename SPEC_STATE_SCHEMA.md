# SPEC_STATE_SCHEMA.md
Version: v1.0.1
Status: LOCKED

Defines canonical account schemas and field semantics for Config/Market/OutcomePool/Position.

## Config
- authority: Pubkey
- oracle: Pubkey
- usdc_mint: Pubkey
- treasury: Pubkey
- treasury_authority: Pubkey
- fee_bps: u16 (0..=MAX_FEE_BPS)
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
- total_pool: u64
- status: enum
- resolved_outcome: Option<u8>
- resolution_payload_hash: [u8;32]
- resolution_timestamp: i64
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
