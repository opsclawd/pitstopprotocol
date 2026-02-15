use anchor_lang::prelude::*;
use crate::constants::MAX_DRIVERS;

#[repr(u8)]
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarketStatus {
    Open = 0,
    Settled = 1,
    Cancelled = 2,
}

/// Market stores the on-chain state for one `(authority, race_id_hash)` market.
#[account]
pub struct Market {
    /// Authority allowed to perform privileged market lifecycle actions.
    pub authority: Pubkey,
    /// Opaque hash identifying the race/event off-chain.
    pub race_id_hash: [u8; 32],
    /// Betting window start (unix timestamp).
    pub open_ts: i64,
    /// Betting window close (unix timestamp).
    pub close_ts: i64,
    /// Serialized MarketStatus value.
    pub status: u8,
    /// Winning driver index once settled, WINNER_UNSET before settlement.
    pub winner_index: u8,
    /// Number of active driver outcomes in this market (<= MAX_DRIVERS).
    pub driver_count: u8,
    /// Fee rate in basis points charged on payout pool.
    pub fee_bps: u16,
    /// Total lamports wagered across all outcomes.
    pub total_pool_lamports: u64,
        /// Fixed-size pools (lamports) per driver index. Unused trailing entries remain 0.
    pub driver_pools_lamports: [u64; MAX_DRIVERS],
    /// Snapshot of winner-side pool captured during settlement.
    pub winner_pool_lamports: u64,
    /// PDA bump used for signer-seed reconstruction.
    pub bump: u8,
}

impl Market {
    /// Byte size used when initializing the Market account.
    /// Keep this in sync with field changes to avoid account allocation bugs.
    pub const INIT_SPACE: usize = 8  // discriminator
        + 32 // authority
        + 32 // race_id_hash
        + 8  // open_ts
        + 8  // close_ts
        + 1  // status
        + 1  // winner_index
        + 1  // driver_count
        + 2  // fee_bps
        + 8  // total_pool_lamports
        + (8 * MAX_DRIVERS) // driver_pools_lamports
        + 8  // winner_pool_lamports
        + 1; // bump
}

/// Position tracks a user's additive bet on a market (single driver in MVP).
#[account]
pub struct Position {
    // NOTE: Position logic lands in issue #3; structure is defined now for schema stability.
    pub user: Pubkey,
    pub market: Pubkey,
    pub driver_index: u8,
    pub amount_lamports: u64,
    pub claimed: bool,
    pub last_bet_ts: i64,
    /// PDA bump used for signer-seed reconstruction.
    pub bump: u8,
}


impl Position {
    /// Byte size used when initializing Position PDA.
    pub const INIT_SPACE: usize = 8  // discriminator
        + 32 // user
        + 32 // market
        + 1  // driver_index
        + 8  // amount_lamports
        + 1  // claimed
        + 8  // last_bet_ts
        + 1; // bump
}
