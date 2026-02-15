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
