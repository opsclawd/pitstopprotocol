use anchor_lang::prelude::*;
use crate::constants::MAX_DRIVERS;

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarketStatus {
    Open = 0,
    Settled = 1,
    Cancelled = 2,
}

#[account]
pub struct Market {
    pub authority: Pubkey,
    pub race_id_hash: [u8; 32],
    pub open_ts: i64,
    pub close_ts: i64,
    pub status: u8,
    pub winner_index: u8,
    pub driver_count: u8,
    pub fee_bps: u16,
    pub total_pool_lamports: u64,
    pub driver_pools_lamports: [u64; MAX_DRIVERS],
    pub winner_pool_lamports: u64,
    pub bump: u8,
}

#[account]
pub struct Position {
    pub user: Pubkey,
    pub market: Pubkey,
    pub driver_index: u8,
    pub amount_lamports: u64,
    pub claimed: bool,
    pub last_bet_ts: i64,
    pub bump: u8,
}
