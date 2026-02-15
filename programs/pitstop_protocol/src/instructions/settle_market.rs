use anchor_lang::prelude::*;

use crate::{
    constants::MAX_DRIVERS,
    errors::PitStopError,
    state::{Market, MarketStatus},
};

/// Accounts required to settle a market.
#[derive(Accounts)]
pub struct SettleMarket<'info> {
    /// Authorized signer (market authority) allowed to settle.
    pub authority: Signer<'info>,

    /// Market PDA to finalize.
    #[account(
        mut,
        has_one = authority,
        seeds = [b"market", market.authority.as_ref(), market.race_id_hash.as_ref()],
        bump = market.bump
    )]
    pub market: Account<'info, Market>,
}

/// settle_market finalizes the market by recording winner and locking status.
pub fn settle_market(ctx: Context<SettleMarket>, winner_index: u8) -> Result<()> {
    let now_ts = Clock::get()?.unix_timestamp;
    let market = &mut ctx.accounts.market;

    require!(
        ctx.accounts.authority.key() == market.authority,
        PitStopError::Unauthorized
    );
    require!(market.status == MarketStatus::Open as u8, PitStopError::MarketNotOpen);
    require!(now_ts >= market.close_ts, PitStopError::SettlementTooEarly);
    require!(winner_index < market.driver_count, PitStopError::InvalidWinnerIndex);

    market.winner_index = winner_index;
    market.winner_pool_lamports = market.driver_pools_lamports[winner_index as usize];
    market.status = MarketStatus::Settled as u8;

    Ok(())
}
