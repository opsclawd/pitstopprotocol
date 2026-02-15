use anchor_lang::prelude::*;

use crate::{
    constants::{BPS_DENOMINATOR, MAX_DRIVERS, WINNER_UNSET},
    errors::PitStopError,
    state::{Market, MarketStatus},
};

/// Accounts required to create a market PDA.
#[derive(Accounts)]
#[instruction(race_id_hash: [u8; 32])]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Market::INIT_SPACE,
        seeds = [b"market", authority.key().as_ref(), race_id_hash.as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,

    pub system_program: Program<'info, System>,
}

/// create_market initializes a fresh Market account with validated configuration.
///
/// Validation is intentionally strict to fail early with explicit error codes.
pub fn create_market(
    ctx: Context<CreateMarket>,
    race_id_hash: [u8; 32],
    open_ts: i64,
    close_ts: i64,
    driver_count: u8,
    fee_bps: u16,
) -> Result<()> {
    require!(
        driver_count >= 2 && driver_count as usize <= MAX_DRIVERS,
        PitStopError::InvalidDriverCount
    );
    require!(close_ts > open_ts, PitStopError::InvalidMarketTimes);
    require!(fee_bps <= BPS_DENOMINATOR, PitStopError::InvalidFeeBps);

        // Populate market with deterministic defaults so later instructions can rely on invariants.
    let market = &mut ctx.accounts.market;
    market.authority = ctx.accounts.authority.key();
    market.race_id_hash = race_id_hash;
    market.open_ts = open_ts;
    market.close_ts = close_ts;
    market.status = MarketStatus::Open as u8;
    market.winner_index = WINNER_UNSET;
    market.driver_count = driver_count;
    market.fee_bps = fee_bps;
    market.total_pool_lamports = 0;
    market.driver_pools_lamports = [0; MAX_DRIVERS];
    market.winner_pool_lamports = 0;
    market.bump = ctx.bumps.market;

    Ok(())
}
