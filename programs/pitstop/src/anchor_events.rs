use anchor_lang::prelude::*;

#[event]
pub struct ConfigInitialized {
    pub authority: Pubkey,
    pub oracle: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury: Pubkey,
    pub fee_bps: u16,
    pub timestamp: i64,
}

#[event]
pub struct MarketCreated {
    pub market: Pubkey,
    pub market_id: [u8; 32],
    pub event_id: [u8; 32],
    pub lock_timestamp: i64,
    pub max_outcomes: u8,
    pub market_type: u8,
    pub rules_version: u16,
    pub timestamp: i64,
}

#[event]
pub struct OutcomeAdded {
    pub market: Pubkey,
    pub outcome_id: u8,
    pub outcome_count: u8,
    pub timestamp: i64,
}

#[event]
pub struct MarketOpened {
    pub market: Pubkey,
    pub timestamp: i64,
}
