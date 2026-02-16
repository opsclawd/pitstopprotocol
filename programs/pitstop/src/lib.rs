//! PitStop protocol Anchor program entrypoint.
//!
//! This crate contains:
//! - Spec/parity instruction logic in `instructions/*` using simple Rust types.
//! - Anchor account + handler wiring that converts between on-chain accounts and
//!   the parity layer while preserving locked spec semantics.

use anchor_lang::prelude::*;

pub mod anchor_accounts;
pub mod anchor_errors;
pub mod anchor_events;

pub mod constants;
pub mod error;
pub mod events;
pub mod math;
pub mod pda;
pub mod state;
pub mod instructions;

pub use anchor_accounts::*;
pub use anchor_errors::*;

// Program id is not yet locked by spec; placeholder is fine for local testing.
declare_id!("6gTRvbxrx2tNGuV8sw4mR7GQ3bDaHLcs6LosjHmw2xcW");

#[program]
pub mod pitstop {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        handlers::initialize(ctx, args)
    }

    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> Result<()> {
        handlers::create_market(ctx, args)
    }

    pub fn add_outcome(ctx: Context<AddOutcome>, args: AddOutcomeArgs) -> Result<()> {
        handlers::add_outcome(ctx, args)
    }

    pub fn finalize_seeding(ctx: Context<FinalizeSeeding>) -> Result<()> {
        handlers::finalize_seeding(ctx)
    }
}

mod handlers {
    use super::*;
    use std::str::FromStr;

    use anchor_spl::token_interface::{Mint, TokenAccount};

    fn clock_unix_timestamp() -> Result<i64> {
        Ok(Clock::get()?.unix_timestamp)
    }

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        // Spec-required validations with protocol errors (not Anchor constraint errors).
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            Pubkey::from_str(constants::REQUIRED_TOKEN_PROGRAM).unwrap(),
            PitStopAnchorError::InvalidTokenProgram
        );

        // Anchor token_interface accounts allow reading decimals even if Token-2022,
        // but we pin via token_program above.
        let usdc_mint: &InterfaceAccount<Mint> = &ctx.accounts.usdc_mint;
        require!(usdc_mint.decimals == 6, PitStopAnchorError::InvalidMintDecimals);

        let treasury: &InterfaceAccount<TokenAccount> = &ctx.accounts.treasury;
        require_keys_eq!(treasury.mint, usdc_mint.key(), PitStopAnchorError::InvalidTreasuryMint);
        require_keys_eq!(
            treasury.owner,
            args.treasury_authority,
            PitStopAnchorError::InvalidTreasuryOwner
        );

        // Call parity layer for remaining validations and canonical field defaults.
        let now_ts = clock_unix_timestamp()?;
        let input = instructions::initialize::InitializeInput {
            authority: ctx.accounts.authority.key().to_string(),
            treasury_authority: args.treasury_authority.to_string(),
            usdc_mint: usdc_mint.key().to_string(),
            treasury: treasury.key().to_string(),
            token_program: ctx.accounts.token_program.key().to_string(),
            usdc_decimals: usdc_mint.decimals,
            treasury_mint: treasury.mint.to_string(),
            treasury_owner: treasury.owner.to_string(),
            max_total_pool_per_market: args.max_total_pool_per_market,
            max_bet_per_user_per_market: args.max_bet_per_user_per_market,
            claim_window_secs: args.claim_window_secs,
            now_ts,
        };

        let (cfg, evt) = instructions::initialize::initialize(input).map_err(PitStopAnchorError::from)?;

        // Persist on-chain config.
        let config = &mut ctx.accounts.config;
        config.authority = ctx.accounts.authority.key();
        config.oracle = ctx.accounts.authority.key();
        config.usdc_mint = usdc_mint.key();
        config.treasury = treasury.key();
        config.treasury_authority = args.treasury_authority;
        config.fee_bps = cfg.fee_bps;
        config.paused = cfg.paused;
        config.max_total_pool_per_market = cfg.max_total_pool_per_market;
        config.max_bet_per_user_per_market = cfg.max_bet_per_user_per_market;
        config.claim_window_secs = cfg.claim_window_secs;
        config.token_program = Pubkey::from_str(constants::REQUIRED_TOKEN_PROGRAM).unwrap();

        emit!(anchor_events::ConfigInitialized {
            authority: ctx.accounts.authority.key(),
            oracle: ctx.accounts.authority.key(),
            usdc_mint: usdc_mint.key(),
            treasury: treasury.key(),
            fee_bps: evt.fee_bps,
            timestamp: now_ts,
        });

        Ok(())
    }

    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );
        require_keys_eq!(ctx.accounts.usdc_mint.key(), ctx.accounts.config.usdc_mint, PitStopAnchorError::InvalidTreasuryMint);

        let now_ts = clock_unix_timestamp()?;
        let input = instructions::create_market::CreateMarketInput {
            authority: ctx.accounts.authority.key().to_string(),
            config_authority: ctx.accounts.config.authority.to_string(),
            token_program: ctx.accounts.token_program.key().to_string(),
            market: ctx.accounts.market.key().to_string(),
            vault: ctx.accounts.vault.key().to_string(),
            market_id: args.market_id,
            event_id: args.event_id,
            lock_timestamp: args.lock_timestamp,
            now_ts,
            max_outcomes: args.max_outcomes,
            market_type: args.market_type,
            rules_version: args.rules_version,
        };

        let (mkt, evt) = instructions::create_market::create_market(input).map_err(PitStopAnchorError::from)?;

        let market = &mut ctx.accounts.market;
        market.market_id = mkt.market_id;
        market.event_id = mkt.event_id;
        market.lock_timestamp = mkt.lock_timestamp;
        market.outcome_count = mkt.outcome_count;
        market.max_outcomes = mkt.max_outcomes;
        market.total_pool = mkt.total_pool;
        market.status = MarketStatus::Seeding;
        market.resolved_outcome = None;
        market.resolution_payload_hash = mkt.resolution_payload_hash;
        market.resolution_timestamp = mkt.resolution_timestamp;
        market.vault = ctx.accounts.vault.key();
        market.market_type = mkt.market_type;
        market.rules_version = mkt.rules_version;

        emit!(anchor_events::MarketCreated {
            market: ctx.accounts.market.key(),
            market_id: evt.market_id,
            event_id: evt.event_id,
            lock_timestamp: evt.lock_timestamp,
            max_outcomes: evt.max_outcomes,
            market_type: evt.market_type,
            rules_version: evt.rules_version,
            timestamp: now_ts,
        });

        Ok(())
    }

    pub fn add_outcome(ctx: Context<AddOutcome>, args: AddOutcomeArgs) -> Result<()> {
        let now_ts = clock_unix_timestamp()?;

        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::add_outcome::AddOutcomeInput {
            authority: ctx.accounts.authority.key().to_string(),
            config_authority: ctx.accounts.config.authority.to_string(),
            market: ctx.accounts.market.key().to_string(),
            market_status: market_state.status,
            market_outcome_count: market_state.outcome_count,
            market_max_outcomes: market_state.max_outcomes,
            outcome_id: args.outcome_id,
            outcome_pool_market: ctx.accounts.market.key().to_string(),
            market_state,
            now_ts,
        };

        let (new_market, _pool, evt) =
            instructions::add_outcome::add_outcome(input).map_err(PitStopAnchorError::from)?;

        // Initialize outcome pool.
        let outcome_pool = &mut ctx.accounts.outcome_pool;
        outcome_pool.market = ctx.accounts.market.key();
        outcome_pool.outcome_id = args.outcome_id;
        outcome_pool.pool_amount = 0;

        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::OutcomeAdded {
            market: ctx.accounts.market.key(),
            outcome_id: evt.outcome_id,
            timestamp: now_ts,
        });

        Ok(())
    }

    pub fn finalize_seeding(ctx: Context<FinalizeSeeding>) -> Result<()> {
        let now_ts = clock_unix_timestamp()?;
        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::finalize_seeding::FinalizeSeedingInput {
            authority: ctx.accounts.authority.key().to_string(),
            config_authority: ctx.accounts.config.authority.to_string(),
            market: ctx.accounts.market.key().to_string(),
            market_status: market_state.status,
            market_outcome_count: market_state.outcome_count,
            market_max_outcomes: market_state.max_outcomes,
            lock_timestamp: market_state.lock_timestamp,
            now_ts,
            market_state,
        };

        let (new_market, _evt) = instructions::finalize_seeding::finalize_seeding(input)
            .map_err(PitStopAnchorError::from)?;
        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::MarketOpened {
            market: ctx.accounts.market.key(),
            timestamp: now_ts,
        });

        Ok(())
    }
}
