#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigInitialized {
    pub authority: String,
    pub oracle: String,
    pub usdc_mint: String,
    pub treasury: String,
    pub fee_bps: u16,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketCreated {
    pub market: String,
    pub market_id: [u8; 32],
    pub event_id: [u8; 32],
    pub lock_timestamp: i64,
    pub max_outcomes: u8,
    pub market_type: u8,
    pub rules_version: u16,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcomeAdded {
    pub market: String,
    pub outcome_id: u8,
    pub outcome_count: u8,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketOpened {
    pub market: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BetPlaced {
    pub market: String,
    pub user: String,
    pub outcome_id: u8,
    pub amount: u64,
    pub market_total_pool: u64,
    pub outcome_pool_amount: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketLocked {
    pub market: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketResolved {
    pub market: String,
    pub winning_outcome: u8,
    pub payload_hash: [u8; 32],
    pub resolution_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketVoided {
    pub market: String,
    pub payload_hash: [u8; 32],
    pub resolution_timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Claimed {
    pub market: String,
    pub user: String,
    pub outcome_id: u8,
    pub payout: u64,
    pub claimed_at: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketSweptEvent {
    pub market: String,
    pub amount: u64,
    pub to_treasury: String,
    pub timestamp: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarketCancelled {
    pub market: String,
    pub timestamp: i64,
}
