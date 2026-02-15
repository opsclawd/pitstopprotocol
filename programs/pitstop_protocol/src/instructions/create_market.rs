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
    // Transaction signer paying rent for the new market account.
    // In MVP this signer also becomes the market authority for privileged actions
    // like settlement/cancellation in later instructions.
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        // Allocate and create a brand new Market PDA.
        init,
        // Authority covers rent/creation cost for this account.
        payer = authority,
        // Exact byte size computed in Market::INIT_SPACE.
        space = Market::INIT_SPACE,
        // Seed strategy intentionally includes authority to avoid global market squatting
        // on a shared race id hash. This makes identity `(authority, race_id_hash)`.
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
    // We require at least 2 outcomes so the market is meaningful, and cap to
    // MAX_DRIVERS so the fixed-size pools array is never indexed out of bounds.
    require!(
        driver_count >= 2 && driver_count as usize <= MAX_DRIVERS,
        PitStopError::InvalidDriverCount
    );

    // Market must close strictly after open. Equal timestamps would create a
    // zero-duration market that can never accept valid bets in practice.
    require!(close_ts > open_ts, PitStopError::InvalidMarketTimes);

    // Basis points are constrained to denominator to avoid invalid fee math.
    require!(fee_bps <= BPS_DENOMINATOR, PitStopError::InvalidFeeBps);

        // Populate market with deterministic defaults so later instructions can rely on invariants.
    let market = &mut ctx.accounts.market;
    // Authority who can perform privileged lifecycle operations later.
    market.authority = ctx.accounts.authority.key();

    // Opaque race identifier hash chosen by the client/off-chain scheduler.
    market.race_id_hash = race_id_hash;

    // Market timing configuration.
    market.open_ts = open_ts;
    market.close_ts = close_ts;

    // Fresh markets always start Open and without a winner.
    market.status = MarketStatus::Open as u8;
    market.winner_index = WINNER_UNSET;

    // Outcome config and fee policy.
    market.driver_count = driver_count;
    market.fee_bps = fee_bps;

    // No funds exist at creation; all pools begin at zero.
    market.total_pool_lamports = 0;
    market.driver_pools_lamports = [0; MAX_DRIVERS];
    market.winner_pool_lamports = 0;

    // Store PDA bump so later CPI/signing flows can reconstruct signer seeds.
    market.bump = ctx.bumps.market;

    Ok(())
}
