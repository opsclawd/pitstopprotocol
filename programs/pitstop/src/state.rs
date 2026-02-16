#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub authority: String,
    pub oracle: String,
    pub usdc_mint: String,
    pub treasury: String,
    pub treasury_authority: String,
    pub fee_bps: u16,
    pub paused: bool,
    pub max_total_pool_per_market: u64,
    pub max_bet_per_user_per_market: u64,
    pub claim_window_secs: i64,
    pub token_program: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarketStatus {
    Seeding,
    Open,
    Locked,
    Resolved,
    Voided,
    Swept,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Market {
    pub market_id: [u8; 32],
    pub event_id: [u8; 32],
    pub lock_timestamp: i64,
    pub outcome_count: u8,
    pub max_outcomes: u8,
    pub total_pool: u64,
    pub status: MarketStatus,
    pub resolved_outcome: Option<u8>,
    pub resolution_payload_hash: [u8; 32],
    pub resolution_timestamp: i64,
    pub vault: String,
    pub market_type: u8,
    pub rules_version: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomePool {
    pub market: String,
    pub outcome_id: u8,
    pub pool_amount: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Position {
    pub market: String,
    pub user: String,
    pub outcome_id: u8,
    pub amount: u64,
    /// Tracks whether the position has been claimed via claim_resolved/claim_voided.
    pub claimed: bool,
    /// Payout recorded at claim time (base units). For resolved losers this is 0.
    pub payout: u64,
}
