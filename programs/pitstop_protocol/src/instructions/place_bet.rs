use anchor_lang::prelude::*;
use anchor_lang::system_program::{self, Transfer};

use crate::{
    constants::{MAX_DRIVERS, MIN_BET_LAMPORTS},
    errors::PitStopError,
    state::{Market, MarketStatus, Position},
};

/// Accounts required to place a bet and aggregate into the user's position.
#[derive(Accounts)]
pub struct PlaceBet<'info> {
    /// Bettor signs and pays both rent (first-time position init) and wager SOL.
    #[account(mut)]
    pub bettor: Signer<'info>,

    /// Existing market PDA that receives SOL escrow and accounting updates.
    #[account(
        mut,
        seeds = [b"market", market.authority.as_ref(), market.race_id_hash.as_ref()],
        bump = market.bump
    )]
    pub market: Account<'info, Market>,

    /// One position PDA per (market, user). Initialized on first bet.
    #[account(
        init_if_needed,
        payer = bettor,
        space = Position::INIT_SPACE,
        seeds = [b"position", market.key().as_ref(), bettor.key().as_ref()],
        bump
    )]
    pub position: Account<'info, Position>,

    pub system_program: Program<'info, System>,
}

/// place_bet transfers SOL into market escrow and updates market/position accounting.
pub fn place_bet(ctx: Context<PlaceBet>, driver_index: u8, amount_lamports: u64) -> Result<()> {
    let now_ts = Clock::get()?.unix_timestamp;
    let market = &mut ctx.accounts.market;
    let position = &mut ctx.accounts.position;

    // Guard rails for bet amount and market lifecycle.
    require!(amount_lamports >= MIN_BET_LAMPORTS, PitStopError::BetTooSmall);
    require!(market.status == MarketStatus::Open as u8, PitStopError::MarketNotOpen);
    // Enforce full betting window: [open_ts, close_ts).
    require!(now_ts >= market.open_ts, PitStopError::BettingNotStarted);
    require!(now_ts < market.close_ts, PitStopError::BettingClosed);

    // Defense-in-depth: market config must stay internally sane.
    require!(
        (market.driver_count as usize) <= MAX_DRIVERS,
        PitStopError::InvalidMarketConfig
    );
    require!(driver_index < market.driver_count, PitStopError::InvalidDriverIndex);

    // Initialize position metadata on first interaction.
    if position.amount_lamports == 0 {
        position.user = ctx.accounts.bettor.key();
        position.market = market.key();
        position.driver_index = driver_index;
        position.claimed = false;
        position.bump = ctx.bumps.position;
    }

    // Defense-in-depth: for existing positions, stored metadata must match PDA identity.
    require!(
        position.user == ctx.accounts.bettor.key() && position.market == market.key(),
        PitStopError::PositionInvariantViolation
    );

    // Enforce add-to-position-only behavior for MVP.
    require!(
        position.driver_index == driver_index,
        PitStopError::DriverSelectionLocked
    );

    // Move wager lamports into the Market PDA account.
    // If transfer fails, tx aborts and no state updates are committed.
    let cpi_ctx = CpiContext::new(
        ctx.accounts.system_program.to_account_info(),
        Transfer {
            from: ctx.accounts.bettor.to_account_info(),
            to: market.to_account_info(),
        },
    );
    system_program::transfer(cpi_ctx, amount_lamports)?;

    // Accounting updates use checked math to preserve consistency.
    position.amount_lamports = position
        .amount_lamports
        .checked_add(amount_lamports)
        .ok_or(PitStopError::MathOverflow)?;
    position.last_bet_ts = now_ts;

    market.total_pool_lamports = market
        .total_pool_lamports
        .checked_add(amount_lamports)
        .ok_or(PitStopError::MathOverflow)?;

    let idx = driver_index as usize;
    market.driver_pools_lamports[idx] = market.driver_pools_lamports[idx]
        .checked_add(amount_lamports)
        .ok_or(PitStopError::MathOverflow)?;

    Ok(())
}
