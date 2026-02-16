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

    pub fn place_bet(ctx: Context<PlaceBet>, args: PlaceBetArgs) -> Result<()> {
        handlers::place_bet(ctx, args)
    }

    pub fn lock_market(ctx: Context<LockMarket>) -> Result<()> {
        handlers::lock_market(ctx)
    }

    pub fn resolve_market(ctx: Context<ResolveMarket>, args: ResolveMarketArgs) -> Result<()> {
        handlers::resolve_market(ctx, args)
    }

    pub fn void_market(ctx: Context<VoidMarket>, args: VoidMarketArgs) -> Result<()> {
        handlers::void_market(ctx, args)
    }

    pub fn claim_resolved(ctx: Context<ClaimResolved>, args: ClaimResolvedArgs) -> Result<()> {
        handlers::claim_resolved(ctx, args)
    }

    pub fn claim_voided(ctx: Context<ClaimVoided>, args: ClaimVoidedArgs) -> Result<()> {
        handlers::claim_voided(ctx, args)
    }

    pub fn sweep_remaining(ctx: Context<SweepRemaining>) -> Result<()> {
        handlers::sweep_remaining(ctx)
    }

    pub fn cancel_market(ctx: Context<CancelMarket>) -> Result<()> {
        handlers::cancel_market(ctx)
    }
}

mod handlers {
    use super::*;
    use std::str::FromStr;

    use anchor_spl::token_interface::{
        close_account, transfer_checked, CloseAccount, Mint, TokenAccount, TransferChecked,
    };

    /// Single clock read helper so all handlers use the same on-chain time source.
    ///
    /// Why this exists:
    /// - Keeps timestamp sourcing consistent across instructions.
    /// - Makes parity-input construction explicit (`now_ts` always comes from Clock).
    fn clock_unix_timestamp() -> Result<i64> {
        Ok(Clock::get()?.unix_timestamp)
    }

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        // Layer 1 validation (Anchor handler level):
        // perform explicit protocol-mapped guards before invoking parity logic.
        //
        // We intentionally map to PitStopAnchorError here so callers see the same
        // deterministic error taxonomy expected by LOCKED specs.
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            Pubkey::from_str(constants::REQUIRED_TOKEN_PROGRAM).unwrap(),
            PitStopAnchorError::InvalidTokenProgram
        );

        // Anchor token_interface lets us read decimals regardless of token flavor,
        // but protocol is currently pinned to USDC(6) + required token program.
        let usdc_mint: &InterfaceAccount<Mint> = &ctx.accounts.usdc_mint;
        require!(usdc_mint.decimals == 6, PitStopAnchorError::InvalidMintDecimals);

        let treasury: &InterfaceAccount<TokenAccount> = &ctx.accounts.treasury;
        require_keys_eq!(treasury.mint, usdc_mint.key(), PitStopAnchorError::InvalidTreasuryMint);
        require_keys_eq!(
            treasury.owner,
            args.treasury_authority,
            PitStopAnchorError::InvalidTreasuryOwner
        );

        // Layer 2 validation (parity/core layer):
        // convert Anchor accounts/args into the pure parity input and let the
        // deterministic spec implementation enforce remaining checks/defaults.
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

        // State commit:
        // parity returned canonical config values; persist those onto Anchor account.
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

        // Event emission:
        // emit after successful state write so off-chain observers see committed transitions.
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
        // Pre-flight account compatibility checks at Anchor boundary.
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );
        require_keys_eq!(ctx.accounts.usdc_mint.key(), ctx.accounts.config.usdc_mint, PitStopAnchorError::InvalidTreasuryMint);

        // Build parity input from Anchor accounts/args.
        // This keeps one authoritative implementation for business rules.
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

        // Commit parity result into on-chain Market account.
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
        // Convert current Market account into parity snapshot, run deterministic
        // add_outcome logic, then write back the resulting state.
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

        // Initialize the newly created outcome_pool PDA.
        // Seeds/space allocation are enforced by the Anchor account context.
        let outcome_pool = &mut ctx.accounts.outcome_pool;
        outcome_pool.market = ctx.accounts.market.key();
        outcome_pool.outcome_id = args.outcome_id;
        outcome_pool.pool_amount = 0;

        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::OutcomeAdded {
            market: ctx.accounts.market.key(),
            outcome_id: evt.outcome_id,
            outcome_count: evt.outcome_count,
            timestamp: now_ts,
        });

        Ok(())
    }

    pub fn finalize_seeding(ctx: Context<FinalizeSeeding>) -> Result<()> {
        // Finalize transition is parity-driven: snapshot -> validate/transition -> commit.
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

    pub fn place_bet(ctx: Context<PlaceBet>, args: PlaceBetArgs) -> Result<()> {
        use anchor_spl::token_interface::{transfer_checked, TransferChecked};

        // Anchor boundary checks for pinned token program + mint/vault relations.
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );
        require_keys_eq!(
            ctx.accounts.usdc_mint.key(),
            ctx.accounts.config.usdc_mint,
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.vault.key(),
            ctx.accounts.market.vault,
            PitStopAnchorError::OutcomeMismatch
        );
        require_keys_eq!(
            ctx.accounts.user_usdc.mint,
            ctx.accounts.usdc_mint.key(),
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.user_usdc.owner,
            ctx.accounts.user.key(),
            PitStopAnchorError::Unauthorized
        );

        // Load/validate outcome_pool with spec-mapped error behavior.
        let expected_pool = Pubkey::find_program_address(
            &[b"outcome", ctx.accounts.market.key().as_ref(), &[args.outcome_id]],
            &crate::id(),
        )
        .0;
        if ctx.accounts.outcome_pool.key() != expected_pool {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }
        if ctx.accounts.outcome_pool.owner != &crate::id() {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }
        let mut outcome_pool: OutcomePool = {
            let data_ref = ctx.accounts.outcome_pool.try_borrow_data()?;
            if data_ref.is_empty() {
                return Err(error!(PitStopAnchorError::OutcomeMismatch));
            }
            let mut slice: &[u8] = &data_ref;
            OutcomePool::try_deserialize(&mut slice)
                .map_err(|_| error!(PitStopAnchorError::OutcomeMismatch))?
        };
        if outcome_pool.market != ctx.accounts.market.key() || outcome_pool.outcome_id != args.outcome_id {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }

        // Initialize position metadata on first creation.
        if ctx.accounts.position.market == Pubkey::default() {
            let pos = &mut ctx.accounts.position;
            pos.market = ctx.accounts.market.key();
            pos.user = ctx.accounts.user.key();
            pos.outcome_id = args.outcome_id;
            pos.amount = 0;
            pos.claimed = false;
            pos.payout = 0;
        }

        let now_ts = clock_unix_timestamp()?;
        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::place_bet::PlaceBetInput {
            config_paused: ctx.accounts.config.paused,
            market_status: market_state.status,
            now_ts,
            market_lock_timestamp: market_state.lock_timestamp,
            outcome_id: args.outcome_id,
            market_outcome_count: market_state.outcome_count,
            market_max_outcomes: market_state.max_outcomes,
            amount: args.amount,
            token_program: ctx.accounts.token_program.key().to_string(),
            outcome_pool_exists: true,
            outcome_pool_market: outcome_pool.market.to_string(),
            outcome_pool_outcome_id: outcome_pool.outcome_id,
            market: ctx.accounts.market.key().to_string(),
            user: ctx.accounts.user.key().to_string(),
            market_total_pool: market_state.total_pool,
            max_total_pool_per_market: ctx.accounts.config.max_total_pool_per_market,
            user_position_amount: ctx.accounts.position.amount,
            max_bet_per_user_per_market: ctx.accounts.config.max_bet_per_user_per_market,
            outcome_pool_amount: outcome_pool.pool_amount,
            vault_amount: ctx.accounts.vault.amount,
            market_state,
            outcome_pool_state: crate::state::OutcomePool {
                market: outcome_pool.market.to_string(),
                outcome_id: outcome_pool.outcome_id,
                pool_amount: outcome_pool.pool_amount,
            },
            position_state: ctx.accounts.position.to_parity(),
        };

        let (new_market, new_pool, new_pos, _new_vault_amount, evt) =
            instructions::place_bet::place_bet(input).map_err(PitStopAnchorError::from)?;

        // Funds move (CPI) happens only after deterministic preconditions pass.
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.user_usdc.to_account_info(),
            mint: ctx.accounts.usdc_mint.to_account_info(),
            to: ctx.accounts.vault.to_account_info(),
            authority: ctx.accounts.user.to_account_info(),
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        transfer_checked(cpi_ctx, args.amount, ctx.accounts.usdc_mint.decimals)?;

        // Commit state updates.
        ctx.accounts.market.apply_parity(&new_market);
        outcome_pool.pool_amount = new_pool.pool_amount;
        {
            let mut data_mut = ctx.accounts.outcome_pool.try_borrow_mut_data()?;
            let mut dst: &mut [u8] = &mut data_mut;
            outcome_pool.try_serialize(&mut dst)?;
        }
        ctx.accounts.position.apply_parity(&new_pos);

        emit!(anchor_events::BetPlaced {
            market: ctx.accounts.market.key(),
            user: ctx.accounts.user.key(),
            outcome_id: evt.outcome_id,
            amount: evt.amount,
            market_total_pool: evt.market_total_pool,
            outcome_pool_amount: evt.outcome_pool_amount,
            timestamp: evt.timestamp,
        });

        Ok(())
    }

    pub fn lock_market(ctx: Context<LockMarket>) -> Result<()> {
        let now_ts = clock_unix_timestamp()?;
        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::lock_market::LockMarketInput {
            authority: ctx.accounts.authority.key().to_string(),
            config_authority: ctx.accounts.config.authority.to_string(),
            market: ctx.accounts.market.key().to_string(),
            market_status: market_state.status,
            now_ts,
            lock_timestamp: market_state.lock_timestamp,
            market_state,
        };

        let (new_market, evt) =
            instructions::lock_market::lock_market(input).map_err(PitStopAnchorError::from)?;
        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::MarketLocked {
            market: ctx.accounts.market.key(),
            timestamp: evt.timestamp,
        });

        Ok(())
    }

    pub fn resolve_market(ctx: Context<ResolveMarket>, args: ResolveMarketArgs) -> Result<()> {
        let now_ts = clock_unix_timestamp()?;

        let expected_pool = Pubkey::find_program_address(
            &[b"outcome", ctx.accounts.market.key().as_ref(), &[args.winning_outcome_id]],
            &crate::id(),
        )
        .0;
        if ctx.accounts.winning_outcome_pool.key() != expected_pool {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }
        if ctx.accounts.winning_outcome_pool.owner != &crate::id() {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }
        let winning_pool: OutcomePool = {
            let data_ref = ctx.accounts.winning_outcome_pool.try_borrow_data()?;
            if data_ref.is_empty() {
                return Err(error!(PitStopAnchorError::OutcomeMismatch));
            }
            let mut slice: &[u8] = &data_ref;
            OutcomePool::try_deserialize(&mut slice)
                .map_err(|_| error!(PitStopAnchorError::OutcomeMismatch))?
        };

        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::resolve_market::ResolveMarketInput {
            oracle: ctx.accounts.oracle.key().to_string(),
            config_oracle: ctx.accounts.config.oracle.to_string(),
            market: ctx.accounts.market.key().to_string(),
            market_state,
            winning_outcome_id: args.winning_outcome_id,
            payload_hash: args.payload_hash,
            winning_outcome_pool_state: Some(crate::state::OutcomePool {
                market: winning_pool.market.to_string(),
                outcome_id: winning_pool.outcome_id,
                pool_amount: winning_pool.pool_amount,
            }),
            now_ts,
        };

        let (new_market, evt) = instructions::resolve_market::resolve_market(input)
            .map_err(PitStopAnchorError::from)?;
        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::MarketResolved {
            market: ctx.accounts.market.key(),
            winning_outcome: evt.winning_outcome,
            payload_hash: evt.payload_hash,
            resolution_timestamp: evt.resolution_timestamp,
        });

        Ok(())
    }

    pub fn void_market(ctx: Context<VoidMarket>, args: VoidMarketArgs) -> Result<()> {
        let now_ts = clock_unix_timestamp()?;
        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::void_market::VoidMarketInput {
            oracle: ctx.accounts.oracle.key().to_string(),
            config_oracle: ctx.accounts.config.oracle.to_string(),
            market: ctx.accounts.market.key().to_string(),
            payload_hash: args.payload_hash,
            now_ts,
            market_state,
        };

        let (new_market, evt) =
            instructions::void_market::void_market(input).map_err(PitStopAnchorError::from)?;
        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::MarketVoided {
            market: ctx.accounts.market.key(),
            payload_hash: evt.payload_hash,
            resolution_timestamp: evt.resolution_timestamp,
        });

        Ok(())
    }

    pub fn claim_resolved(ctx: Context<ClaimResolved>, args: ClaimResolvedArgs) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );
        require_keys_eq!(
            ctx.accounts.usdc_mint.key(),
            ctx.accounts.config.usdc_mint,
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.vault.key(),
            ctx.accounts.market.vault,
            PitStopAnchorError::OutcomeMismatch
        );
        require_keys_eq!(
            ctx.accounts.user_usdc.mint,
            ctx.accounts.usdc_mint.key(),
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.user_usdc.owner,
            ctx.accounts.user.key(),
            PitStopAnchorError::Unauthorized
        );

        let expected_pool = Pubkey::find_program_address(
            &[OUTCOME_SEED, ctx.accounts.market.key().as_ref(), &[args.outcome_id]],
            &crate::id(),
        )
        .0;
        if ctx.accounts.outcome_pool.key() != expected_pool {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }
        if ctx.accounts.outcome_pool.owner != &crate::id() {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }
        let outcome_pool: OutcomePool = {
            let data_ref = ctx.accounts.outcome_pool.try_borrow_data()?;
            if data_ref.is_empty() {
                return Err(error!(PitStopAnchorError::OutcomeMismatch));
            }
            let mut slice: &[u8] = &data_ref;
            OutcomePool::try_deserialize(&mut slice)
                .map_err(|_| error!(PitStopAnchorError::OutcomeMismatch))?
        };
        if outcome_pool.market != ctx.accounts.market.key() || outcome_pool.outcome_id != args.outcome_id {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }

        let now_ts = clock_unix_timestamp()?;
        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::claim_resolved::ClaimResolvedInput {
            market: ctx.accounts.market.key().to_string(),
            user: ctx.accounts.user.key().to_string(),
            market_status: market_state.status,
            now_ts,
            resolution_timestamp: market_state.resolution_timestamp,
            claim_window_secs: ctx.accounts.config.claim_window_secs,
            fee_bps: ctx.accounts.config.fee_bps,
            resolved_outcome: market_state.resolved_outcome,
            outcome_id: args.outcome_id,
            position_claimed: ctx.accounts.position.claimed,
            position_amount: ctx.accounts.position.amount,
            outcome_pool_exists: true,
            outcome_pool_market: outcome_pool.market.to_string(),
            outcome_pool_outcome_id: outcome_pool.outcome_id,
            outcome_pool_amount: outcome_pool.pool_amount,
            vault_amount: ctx.accounts.vault.amount,
            user_usdc_amount: ctx.accounts.user_usdc.amount,
            market_state,
            outcome_pool_state: crate::state::OutcomePool {
                market: outcome_pool.market.to_string(),
                outcome_id: outcome_pool.outcome_id,
                pool_amount: outcome_pool.pool_amount,
            },
            position_state: ctx.accounts.position.to_parity(),
        };

        let (new_pos, _new_vault_amount, _new_user_amount, evt) =
            instructions::claim_resolved::claim_resolved(input).map_err(PitStopAnchorError::from)?;

        if evt.payout > 0 {
            let cpi_accounts = TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.usdc_mint.to_account_info(),
                to: ctx.accounts.user_usdc.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            };
            let (_market_pda, market_bump) = Pubkey::find_program_address(
                &[MARKET_SEED, ctx.accounts.market.market_id.as_ref()],
                &crate::id(),
            );
            let signer_seeds: &[&[u8]] = &[
                MARKET_SEED,
                ctx.accounts.market.market_id.as_ref(),
                &[market_bump],
            ];
            let signer = &[signer_seeds];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            );
            transfer_checked(cpi_ctx, evt.payout, ctx.accounts.usdc_mint.decimals)?;
        }

        ctx.accounts.position.apply_parity(&new_pos);

        emit!(anchor_events::Claimed {
            market: ctx.accounts.market.key(),
            user: ctx.accounts.user.key(),
            outcome_id: evt.outcome_id,
            payout: evt.payout,
            claimed_at: evt.claimed_at,
        });

        Ok(())
    }

    pub fn claim_voided(ctx: Context<ClaimVoided>, args: ClaimVoidedArgs) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );
        require_keys_eq!(
            ctx.accounts.usdc_mint.key(),
            ctx.accounts.config.usdc_mint,
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.vault.key(),
            ctx.accounts.market.vault,
            PitStopAnchorError::OutcomeMismatch
        );
        require_keys_eq!(
            ctx.accounts.user_usdc.mint,
            ctx.accounts.usdc_mint.key(),
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.user_usdc.owner,
            ctx.accounts.user.key(),
            PitStopAnchorError::Unauthorized
        );

        let now_ts = clock_unix_timestamp()?;
        let input = instructions::claim_voided::ClaimVoidedInput {
            market: ctx.accounts.market.key().to_string(),
            user: ctx.accounts.user.key().to_string(),
            market_status: ctx.accounts.market.to_parity().status,
            resolution_timestamp: ctx.accounts.market.resolution_timestamp,
            claim_window_secs: ctx.accounts.config.claim_window_secs,
            now_ts,
            outcome_id: args.outcome_id,
            user_usdc_amount: ctx.accounts.user_usdc.amount,
            vault_amount: ctx.accounts.vault.amount,
            position_state: ctx.accounts.position.to_parity(),
        };

        let (new_pos, _new_user_amount, _new_vault_amount, evt) =
            instructions::claim_voided::claim_voided(input).map_err(PitStopAnchorError::from)?;

        if evt.payout > 0 {
            let cpi_accounts = TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.usdc_mint.to_account_info(),
                to: ctx.accounts.user_usdc.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            };
            let (_market_pda, market_bump) = Pubkey::find_program_address(
                &[MARKET_SEED, ctx.accounts.market.market_id.as_ref()],
                &crate::id(),
            );
            let signer_seeds: &[&[u8]] = &[
                MARKET_SEED,
                ctx.accounts.market.market_id.as_ref(),
                &[market_bump],
            ];
            let signer = &[signer_seeds];
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            );
            transfer_checked(cpi_ctx, evt.payout, ctx.accounts.usdc_mint.decimals)?;
        }

        ctx.accounts.position.apply_parity(&new_pos);

        emit!(anchor_events::Claimed {
            market: ctx.accounts.market.key(),
            user: ctx.accounts.user.key(),
            outcome_id: evt.outcome_id,
            payout: evt.payout,
            claimed_at: evt.claimed_at,
        });

        Ok(())
    }

    pub fn sweep_remaining(ctx: Context<SweepRemaining>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );
        require_keys_eq!(
            ctx.accounts.usdc_mint.key(),
            ctx.accounts.config.usdc_mint,
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.treasury.mint,
            ctx.accounts.usdc_mint.key(),
            PitStopAnchorError::InvalidTreasuryMint
        );
        require_keys_eq!(
            ctx.accounts.treasury.owner,
            ctx.accounts.config.treasury_authority,
            PitStopAnchorError::InvalidTreasuryOwner
        );
        require_keys_eq!(
            ctx.accounts.vault.key(),
            ctx.accounts.market.vault,
            PitStopAnchorError::OutcomeMismatch
        );

        let now_ts = clock_unix_timestamp()?;
        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::sweep_remaining::SweepRemainingInput {
            authority: ctx.accounts.authority.key().to_string(),
            config_authority: ctx.accounts.config.authority.to_string(),
            market: ctx.accounts.market.key().to_string(),
            now_ts,
            claim_window_secs: ctx.accounts.config.claim_window_secs,
            token_program: ctx.accounts.token_program.key().to_string(),
            treasury: ctx.accounts.treasury.key().to_string(),
            config_treasury: ctx.accounts.config.treasury.to_string(),
            treasury_mint: ctx.accounts.treasury.mint.to_string(),
            usdc_mint: ctx.accounts.usdc_mint.key().to_string(),
            treasury_owner: ctx.accounts.treasury.owner.to_string(),
            treasury_authority: ctx.accounts.config.treasury_authority.to_string(),
            vault_amount: ctx.accounts.vault.amount,
            treasury_amount: ctx.accounts.treasury.amount,
            market_state,
        };

        let (new_market, _new_treasury_amount, swept_amount, _vault_closed, _vault_exists, _used_seeds, evt) =
            instructions::sweep_remaining::sweep_remaining(input).map_err(PitStopAnchorError::from)?;

        let (_market_pda, market_bump) = Pubkey::find_program_address(
            &[MARKET_SEED, ctx.accounts.market.market_id.as_ref()],
            &crate::id(),
        );
        let signer_seeds: &[&[u8]] = &[
            MARKET_SEED,
            ctx.accounts.market.market_id.as_ref(),
            &[market_bump],
        ];
        let signer = &[signer_seeds];

        if swept_amount > 0 {
            let transfer_accounts = TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.usdc_mint.to_account_info(),
                to: ctx.accounts.treasury.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            };
            let transfer_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                transfer_accounts,
                signer,
            );
            transfer_checked(transfer_ctx, swept_amount, ctx.accounts.usdc_mint.decimals)?;
        }

        let close_accounts = CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.close_destination.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        };
        let close_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_accounts,
            signer,
        );
        close_account(close_ctx)?;

        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::MarketSweptEvent {
            market: ctx.accounts.market.key(),
            amount: evt.amount,
            to_treasury: ctx.accounts.treasury.key(),
            timestamp: evt.timestamp,
        });

        Ok(())
    }

    pub fn cancel_market(ctx: Context<CancelMarket>) -> Result<()> {
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );
        require_keys_eq!(
            ctx.accounts.vault.key(),
            ctx.accounts.market.vault,
            PitStopAnchorError::OutcomeMismatch
        );

        let now_ts = clock_unix_timestamp()?;
        let input = instructions::cancel_market::CancelMarketInput {
            authority: ctx.accounts.authority.key().to_string(),
            config_authority: ctx.accounts.config.authority.to_string(),
            close_destination: ctx.accounts.close_destination.key().to_string(),
            market: ctx.accounts.market.key().to_string(),
            market_status: ctx.accounts.market.to_parity().status,
            now_ts,
            lock_timestamp: ctx.accounts.market.lock_timestamp,
            market_state: ctx.accounts.market.to_parity(),
            vault_amount: ctx.accounts.vault.amount,
        };

        let (new_market, evt) =
            instructions::cancel_market::cancel_market(input).map_err(PitStopAnchorError::from)?;

        let (_market_pda, market_bump) = Pubkey::find_program_address(
            &[MARKET_SEED, ctx.accounts.market.market_id.as_ref()],
            &crate::id(),
        );
        let signer_seeds: &[&[u8]] = &[
            MARKET_SEED,
            ctx.accounts.market.market_id.as_ref(),
            &[market_bump],
        ];
        let signer = &[signer_seeds];
        let close_accounts = CloseAccount {
            account: ctx.accounts.vault.to_account_info(),
            destination: ctx.accounts.close_destination.to_account_info(),
            authority: ctx.accounts.market.to_account_info(),
        };
        let close_ctx = CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            close_accounts,
            signer,
        );
        close_account(close_ctx)?;

        ctx.accounts.market.apply_parity(&new_market);

        emit!(anchor_events::MarketCancelled {
            market: ctx.accounts.market.key(),
            timestamp: evt.timestamp,
        });

        Ok(())
    }
}
