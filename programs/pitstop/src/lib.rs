//! PitStop protocol Anchor program entrypoint.
//!
//! This file is the **wiring layer** between:
//! - Anchor (accounts, CPI helpers, event emission), and
//! - the spec/parity “core” logic (pure Rust, deterministic, spec-mapped errors).
//!
//! In general, every instruction follows the same pattern:
//! 1) Do minimal **Anchor-boundary** checks (account relationships, pinned token program).
//! 2) Build a **parity input struct** (plain Rust types, mostly strings/ints).
//! 3) Call `instructions::<ix>::<ix>(input)` to run the spec logic.
//! 4) Commit returned state into Anchor accounts.
//! 5) Emit an Anchor event.

use anchor_lang::prelude::*; // Anchor prelude: Result, Context, require!, emit!, Pubkey, etc.

// Re-exported modules that define Anchor account types, errors, events, constants, and parity wiring.
pub mod anchor_accounts; // Anchor `#[account]` types + `#[derive(Accounts)]` instruction contexts.
pub mod anchor_errors; // Anchor-facing error enum + mappings from parity errors.
pub mod anchor_events; // Anchor `#[event]` structs.

pub mod constants; // Hardcoded constants (e.g., required token program id).
pub mod error; // Shared/parity error types.
pub mod events; // Parity/core event types.
pub mod instructions;
pub mod math; // Math helpers used by parity layer.
pub mod pda; // PDA helper utilities.
pub mod state; // Parity/core state structs + conversions. // Parity/core instruction implementations (spec-driven).

// Convenience exports so other modules can `use crate::*` for key types.
pub use anchor_accounts::*; // Export account/context types at crate root.
pub use anchor_errors::*; // Export Anchor error type(s) at crate root.

// Program id for this Anchor program.
// NOTE: Program id is not yet locked by spec; this placeholder is fine for local testing.
declare_id!("6gTRvbxrx2tNGuV8sw4mR7GQ3bDaHLcs6LosjHmw2xcW");

// Anchor program module: every function here is a Solana instruction entrypoint.
#[program]
pub mod pitstop {
    use super::*; // Pull crate-level types and modules into scope.

    // Initialize the protocol config (authority, mint/treasury wiring, limits).
    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        handlers::initialize(ctx, args) // Delegate to handler for implementation.
    }

    // Create a new market (vault + metadata + rules version).
    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> Result<()> {
        handlers::create_market(ctx, args) // Delegate to handler.
    }

    // Add a new outcome (creates an OutcomePool PDA for that outcome).
    pub fn add_outcome(ctx: Context<AddOutcome>, args: AddOutcomeArgs) -> Result<()> {
        handlers::add_outcome(ctx, args) // Delegate to handler.
    }

    // Finalize seeding, transitioning market into the open/betting state.
    pub fn finalize_seeding(ctx: Context<FinalizeSeeding>) -> Result<()> {
        handlers::finalize_seeding(ctx) // Delegate to handler.
    }

    // Place a bet into a given outcome pool (transfers USDC into the market vault).
    pub fn place_bet(ctx: Context<PlaceBet>, args: PlaceBetArgs) -> Result<()> {
        handlers::place_bet(ctx, args) // Delegate to handler.
    }

    // Lock the market (stop accepting bets) once lock time is reached.
    pub fn lock_market(ctx: Context<LockMarket>) -> Result<()> {
        handlers::lock_market(ctx) // Delegate to handler.
    }

    // Resolve the market with a winning outcome id (oracle action).
    pub fn resolve_market(ctx: Context<ResolveMarket>, args: ResolveMarketArgs) -> Result<()> {
        handlers::resolve_market(ctx, args) // Delegate to handler.
    }

    // Void the market (oracle action), allowing bettors to reclaim principal.
    pub fn void_market(ctx: Context<VoidMarket>, args: VoidMarketArgs) -> Result<()> {
        handlers::void_market(ctx, args) // Delegate to handler.
    }

    // Claim payout after resolution (winners only; applies protocol fee).
    pub fn claim_resolved(ctx: Context<ClaimResolved>, args: ClaimResolvedArgs) -> Result<()> {
        handlers::claim_resolved(ctx, args) // Delegate to handler.
    }

    // Claim principal back after a market is voided.
    pub fn claim_voided(ctx: Context<ClaimVoided>, args: ClaimVoidedArgs) -> Result<()> {
        handlers::claim_voided(ctx, args) // Delegate to handler.
    }

    // Sweep remaining funds after claim window ends (to treasury) and close vault.
    pub fn sweep_remaining(ctx: Context<SweepRemaining>) -> Result<()> {
        handlers::sweep_remaining(ctx) // Delegate to handler.
    }

    // Cancel a market early (admin action), closing vault and marking cancelled.
    pub fn cancel_market(ctx: Context<CancelMarket>) -> Result<()> {
        handlers::cancel_market(ctx) // Delegate to handler.
    }
}

// Private module containing the actual instruction implementations.
// We keep these out of the `#[program]` module to make code organization clearer.
mod handlers {
    use super::*; // Import crate-level items.

    use std::str::FromStr; // Used to parse Pubkey from constant string.

    // Token-2022 / SPL token-interface CPI helpers and account types.
    use anchor_spl::token_interface::{
        close_account,    // CPI helper to close a token account.
        transfer_checked, // CPI helper to transfer tokens with mint-decimal checks.
        CloseAccount,     // CPI accounts struct for closing.
        Mint,             // Token mint interface account.
        TokenAccount,     // Token account interface account.
        TransferChecked,  // CPI accounts struct for checked transfer.
    };

    /// Single clock read helper so all handlers use the same on-chain time source.
    ///
    /// Why this exists:
    /// - Keeps timestamp sourcing consistent across instructions.
    /// - Makes parity-input construction explicit (`now_ts` always comes from Clock).
    fn clock_unix_timestamp() -> Result<i64> {
        Ok(Clock::get()?.unix_timestamp) // Read sysvar clock and return UNIX timestamp.
    }

    /// Parse (and validate) the configured required token program pubkey.
    ///
    /// If the constant is malformed, we map to a deterministic Anchor error.
    fn required_token_program_pubkey() -> Result<Pubkey> {
        Pubkey::from_str(constants::REQUIRED_TOKEN_PROGRAM) // Parse the string constant.
            .map_err(|_| error!(PitStopAnchorError::InvalidTokenProgram)) // Map parse failure.
    }

    /// Canonical OutcomePool loader used by all handlers that need deterministic
    /// `OutcomeMismatch` mapping for missing/wrong/malformed pool accounts.
    ///
    /// This function does three things:
    /// 1) Re-derives the expected PDA for (market, outcome_id) and checks the key matches.
    /// 2) Verifies ownership is this program.
    /// 3) Deserializes `OutcomePool` and validates its fields.
    fn load_outcome_pool_checked(
        pool_account: &AccountInfo, // The raw account info passed by the client.
        market: Pubkey,             // The market pubkey we expect this pool to belong to.
        outcome_id: u8,             // The outcome id we expect this pool to represent.
    ) -> Result<OutcomePool> {
        // Derive the PDA we *expect* for this outcome pool.
        let expected_pool = Pubkey::find_program_address(
            &[OUTCOME_SEED, market.as_ref(), &[outcome_id]], // Seeds: "outcome" + market + outcome_id.
            &crate::id(),                                    // Program id for PDA derivation.
        )
        .0; // Take the derived address (ignore bump here).

        // If the provided account key is not the expected PDA, map to OutcomeMismatch.
        if pool_account.key() != expected_pool {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }

        // Ensure the PDA is owned by this program (so we can deserialize it as our type).
        if pool_account.owner != &crate::id() {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }

        // Borrow the raw account data bytes.
        let data_ref = pool_account.try_borrow_data()?;

        // OutcomePool is an Anchor account, so it must have at least an 8-byte discriminator.
        if data_ref.len() < 8 {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }

        // Deserialize the account data into an OutcomePool struct.
        let mut slice: &[u8] = &data_ref; // Anchor deserializer consumes a byte slice cursor.
        let pool = OutcomePool::try_deserialize(&mut slice) // Try to deserialize.
            .map_err(|_| error!(PitStopAnchorError::OutcomeMismatch))?; // Map errors deterministically.

        // Final sanity check: ensure the stored fields match the (market, outcome_id) we expect.
        if pool.market != market || pool.outcome_id != outcome_id {
            return Err(error!(PitStopAnchorError::OutcomeMismatch));
        }

        Ok(pool) // Return the validated pool.
    }

    // -------------------------
    // Instruction handlers
    // -------------------------

    pub fn initialize(ctx: Context<Initialize>, args: InitializeArgs) -> Result<()> {
        // Layer 1 validation (Anchor handler level):
        // perform explicit protocol-mapped guards before invoking parity logic.
        //
        // We intentionally map to PitStopAnchorError here so callers see the same
        // deterministic error taxonomy expected by LOCKED specs.

        // Require that the token program passed matches the pinned required token program.
        require_keys_eq!(
            ctx.accounts.token_program.key(), // Actual token program account provided.
            constants::REQUIRED_TOKEN_PROGRAM_ID, // Required token program id.
            PitStopAnchorError::InvalidTokenProgram  // Error if mismatch.
        );

        // Anchor token_interface lets us read decimals regardless of token flavor,
        // but protocol is currently pinned to USDC(6) + required token program.
        let usdc_mint: &InterfaceAccount<Mint> = &ctx.accounts.usdc_mint; // USDC mint interface.
        require!(
            usdc_mint.decimals == 6,
            PitStopAnchorError::InvalidMintDecimals
        ); // Enforce 6 decimals.

        // Validate treasury token account wiring.
        let treasury: &InterfaceAccount<TokenAccount> = &ctx.accounts.treasury; // Treasury token account.
        require_keys_eq!(
            treasury.mint,
            usdc_mint.key(),
            PitStopAnchorError::InvalidTreasuryMint
        ); // Treasury must hold USDC.
        require_keys_eq!(
            treasury.owner,                           // Token account owner.
            args.treasury_authority,                  // Expected authority (passed in args).
            PitStopAnchorError::InvalidTreasuryOwner  // Error if mismatch.
        );

        // Layer 2 validation (parity/core layer):
        // convert Anchor accounts/args into the pure parity input and let the
        // deterministic spec implementation enforce remaining checks/defaults.

        // Read current chain time.
        let now_ts = clock_unix_timestamp()?;

        // Build the parity input struct (strings for pubkeys to match spec fixtures).
        let input = instructions::initialize::InitializeInput {
            authority: ctx.accounts.authority.key().to_string(), // Config authority.
            treasury_authority: args.treasury_authority.to_string(), // Who controls treasury account.
            usdc_mint: usdc_mint.key().to_string(),                  // USDC mint.
            treasury: treasury.key().to_string(), // Treasury token account address.
            token_program: ctx.accounts.token_program.key().to_string(), // Token program address.
            usdc_decimals: usdc_mint.decimals,    // Mint decimals (should be 6).
            treasury_mint: treasury.mint.to_string(), // Treasury mint field.
            treasury_owner: treasury.owner.to_string(), // Treasury owner field.
            max_total_pool_per_market: args.max_total_pool_per_market, // Risk control cap.
            max_bet_per_user_per_market: args.max_bet_per_user_per_market, // Per-user cap.
            claim_window_secs: args.claim_window_secs, // Claim window after resolution.
            now_ts,                               // Current timestamp.
        };

        // Run parity logic; map any parity error into Anchor error surface.
        let (cfg, evt) =
            instructions::initialize::initialize(input).map_err(PitStopAnchorError::from)?;

        // State commit:
        // parity returned canonical config values; persist those onto Anchor account.
        let config = &mut ctx.accounts.config; // Mutable reference to the on-chain Config account.
        config.authority = ctx.accounts.authority.key(); // Set authority.
        config.oracle = ctx.accounts.authority.key(); // Default oracle to authority for MVP.
        config.usdc_mint = usdc_mint.key(); // Store USDC mint.
        config.treasury = treasury.key(); // Store treasury account.
        config.treasury_authority = args.treasury_authority; // Store treasury authority.
        config.fee_bps = cfg.fee_bps; // Store fee bps from parity.
        config.paused = cfg.paused; // Store paused flag.
        config.max_total_pool_per_market = cfg.max_total_pool_per_market; // Store cap.
        config.max_bet_per_user_per_market = cfg.max_bet_per_user_per_market; // Store per-user cap.
        config.claim_window_secs = cfg.claim_window_secs; // Store claim window.
        config.token_program = constants::REQUIRED_TOKEN_PROGRAM_ID; // Pin token program.

        // Event emission:
        // emit after successful state write so off-chain observers see committed transitions.
        emit!(anchor_events::ConfigInitialized {
            authority: ctx.accounts.authority.key(), // Authority in event.
            oracle: ctx.accounts.authority.key(),    // Oracle in event.
            usdc_mint: usdc_mint.key(),              // Mint in event.
            treasury: treasury.key(),                // Treasury in event.
            fee_bps: evt.fee_bps,                    // Fee bps from parity event.
            timestamp: now_ts,                       // Timestamp.
        });

        Ok(()) // Signal success.
    }

    pub fn create_market(ctx: Context<CreateMarket>, args: CreateMarketArgs) -> Result<()> {
        // Pre-flight account compatibility checks at Anchor boundary.

        // Token program must match config’s pinned token program.
        require_keys_eq!(
            ctx.accounts.token_program.key(),
            ctx.accounts.config.token_program,
            PitStopAnchorError::InvalidTokenProgram
        );

        // Taxonomy note: current LOCKED error surface reuses InvalidTreasuryMint
        // for config-usdc mint mismatches at this boundary.
        require_keys_eq!(
            ctx.accounts.usdc_mint.key(),
            ctx.accounts.config.usdc_mint,
            PitStopAnchorError::InvalidTreasuryMint
        );

        // Build parity input from Anchor accounts/args.
        // This keeps one authoritative implementation for business rules.
        let now_ts = clock_unix_timestamp()?; // Read time.
        let input = instructions::create_market::CreateMarketInput {
            authority: ctx.accounts.authority.key().to_string(), // Caller authority.
            config_authority: ctx.accounts.config.authority.to_string(), // Stored config authority.
            token_program: ctx.accounts.token_program.key().to_string(), // Token program id.
            market: ctx.accounts.market.key().to_string(),       // New market PDA.
            vault: ctx.accounts.vault.key().to_string(),         // Vault token account.
            market_id: args.market_id,                           // User-supplied market id bytes.
            event_id: args.event_id,                             // External event id bytes.
            lock_timestamp: args.lock_timestamp,                 // When market should lock.
            now_ts,                                              // Current time.
            max_outcomes: args.max_outcomes,                     // Outcome capacity.
            market_type: args.market_type,                       // Market type enum.
            rules_version: args.rules_version,                   // Rules version bytes.
        };

        // Run parity create_market.
        let (mkt, evt) =
            instructions::create_market::create_market(input).map_err(PitStopAnchorError::from)?;

        // Commit parity result into on-chain Market account.
        let market = &mut ctx.accounts.market; // Mutable market account.
        market.market_id = mkt.market_id; // Persist id.
        market.event_id = mkt.event_id; // Persist event id.
        market.lock_timestamp = mkt.lock_timestamp; // Persist lock time.
        market.outcome_count = mkt.outcome_count; // Persist current outcome count.
        market.max_outcomes = mkt.max_outcomes; // Persist max outcomes.
        market.total_pool = mkt.total_pool; // Persist total pool.
        market.status = MarketStatus::Seeding; // New markets start in seeding.
        market.resolved_outcome = None; // Not resolved.
        market.resolution_payload_hash = mkt.resolution_payload_hash; // Persist hash.
        market.resolution_timestamp = mkt.resolution_timestamp; // Persist resolution time (None).
        market.vault = ctx.accounts.vault.key(); // Store vault address.
        market.market_type = mkt.market_type; // Store type.
        market.rules_version = mkt.rules_version; // Store rules version.

        // Emit market created event.
        emit!(anchor_events::MarketCreated {
            market: ctx.accounts.market.key(),  // Market PDA.
            market_id: evt.market_id,           // Market id bytes.
            event_id: evt.event_id,             // Event id bytes.
            lock_timestamp: evt.lock_timestamp, // Lock time.
            max_outcomes: evt.max_outcomes,     // Max outcomes.
            market_type: evt.market_type,       // Type.
            rules_version: evt.rules_version,   // Rules version.
            timestamp: now_ts,                  // Timestamp.
        });

        Ok(()) // Success.
    }

    pub fn add_outcome(ctx: Context<AddOutcome>, args: AddOutcomeArgs) -> Result<()> {
        // Convert current Market account into parity snapshot, run deterministic
        // add_outcome logic, then write back the resulting state.

        let now_ts = clock_unix_timestamp()?; // Read time.

        let market_state = ctx.accounts.market.to_parity(); // Snapshot market into parity struct.
        let input = instructions::add_outcome::AddOutcomeInput {
            authority: ctx.accounts.authority.key().to_string(), // Caller authority.
            config_authority: ctx.accounts.config.authority.to_string(), // Config authority.
            market: ctx.accounts.market.key().to_string(),       // Market pubkey.
            market_status: market_state.status,                  // Current status.
            market_outcome_count: market_state.outcome_count,    // Current outcomes.
            market_max_outcomes: market_state.max_outcomes,      // Max outcomes.
            outcome_id: args.outcome_id,                         // New outcome id.
            outcome_pool_market: ctx.accounts.market.key().to_string(), // Parity expects strings.
            market_state,                                        // Full market snapshot.
            now_ts,                                              // Time.
        };

        // Run parity add_outcome.
        let (new_market, _pool, evt) =
            instructions::add_outcome::add_outcome(input).map_err(PitStopAnchorError::from)?;

        // Initialize the newly created outcome_pool PDA.
        // Seeds/space allocation are enforced by the Anchor account context.
        let outcome_pool = &mut ctx.accounts.outcome_pool; // Mutable pool account.
        outcome_pool.market = ctx.accounts.market.key(); // Store market.
        outcome_pool.outcome_id = args.outcome_id; // Store outcome id.
        outcome_pool.pool_amount = 0; // Start at zero liquidity.

        ctx.accounts.market.apply_parity(&new_market); // Write updated market fields back.

        // Emit outcome added event.
        emit!(anchor_events::OutcomeAdded {
            market: ctx.accounts.market.key(), // Market.
            outcome_id: evt.outcome_id,        // Outcome id.
            outcome_count: evt.outcome_count,  // New count.
            timestamp: now_ts,                 // Time.
        });

        Ok(()) // Success.
    }

    pub fn finalize_seeding(ctx: Context<FinalizeSeeding>) -> Result<()> {
        // Finalize transition is parity-driven: snapshot -> validate/transition -> commit.

        let now_ts = clock_unix_timestamp()?; // Read time.
        let market_state = ctx.accounts.market.to_parity(); // Snapshot market.
        let input = instructions::finalize_seeding::FinalizeSeedingInput {
            authority: ctx.accounts.authority.key().to_string(), // Caller.
            config_authority: ctx.accounts.config.authority.to_string(), // Config authority.
            market: ctx.accounts.market.key().to_string(),       // Market.
            market_status: market_state.status,                  // Status.
            market_outcome_count: market_state.outcome_count,    // Outcome count.
            market_max_outcomes: market_state.max_outcomes,      // Max outcomes.
            lock_timestamp: market_state.lock_timestamp,         // Lock time.
            now_ts,                                              // Time.
            market_state,                                        // Full snapshot.
        };

        // Run parity finalize.
        let (new_market, _evt) = instructions::finalize_seeding::finalize_seeding(input)
            .map_err(PitStopAnchorError::from)?;

        ctx.accounts.market.apply_parity(&new_market); // Commit market transition.

        // Emit opened event.
        emit!(anchor_events::MarketOpened {
            market: ctx.accounts.market.key(),
            timestamp: now_ts,
        });

        Ok(())
    }

    pub fn place_bet(ctx: Context<PlaceBet>, args: PlaceBetArgs) -> Result<()> {
        // NOTE: We import CPI helpers locally as well to keep the handler self-contained.
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

        // Load/validate outcome_pool with spec-mapped deterministic behavior.
        let mut outcome_pool = load_outcome_pool_checked(
            &ctx.accounts.outcome_pool,
            ctx.accounts.market.key(),
            args.outcome_id,
        )?;

        // Initialize position metadata on first creation.
        // Anchor created the PDA account, but its fields may still be default.
        if ctx.accounts.position.market == Pubkey::default() {
            let pos = &mut ctx.accounts.position; // Mutable position account.
            pos.market = ctx.accounts.market.key(); // Set market.
            pos.user = ctx.accounts.user.key(); // Set user.
            pos.outcome_id = args.outcome_id; // Set outcome.
            pos.amount = 0; // Start amount.
            pos.claimed = false; // Not claimed.
            pos.payout = 0; // No payout computed yet.
        }

        // Build parity input.
        let now_ts = clock_unix_timestamp()?; // Read time.
        let market_state = ctx.accounts.market.to_parity(); // Snapshot market.
        let input = instructions::place_bet::PlaceBetInput {
            config_paused: ctx.accounts.config.paused, // Global paused flag.
            market_status: market_state.status,        // Must be open.
            now_ts,                                    // Time.
            market_lock_timestamp: market_state.lock_timestamp, // Used to enforce lock.
            outcome_id: args.outcome_id,               // Outcome.
            market_outcome_count: market_state.outcome_count, // Count.
            market_max_outcomes: market_state.max_outcomes, // Max.
            amount: args.amount,                       // Bet amount.
            token_program: ctx.accounts.token_program.key().to_string(), // Token program id.
            outcome_pool_exists: true,                 // Anchor context ensured it exists.
            outcome_pool_market: outcome_pool.market.to_string(), // Pool market.
            outcome_pool_outcome_id: outcome_pool.outcome_id, // Pool outcome.
            market: ctx.accounts.market.key().to_string(), // Market pk.
            user: ctx.accounts.user.key().to_string(), // User pk.
            market_total_pool: market_state.total_pool, // Total pool.
            max_total_pool_per_market: ctx.accounts.config.max_total_pool_per_market, // Cap.
            user_position_amount: ctx.accounts.position.amount, // Existing position amount.
            max_bet_per_user_per_market: ctx.accounts.config.max_bet_per_user_per_market, // Per-user cap.
            outcome_pool_amount: outcome_pool.pool_amount, // Current outcome pool.
            vault_amount: ctx.accounts.vault.amount,       // Current vault token balance.
            market_state,                                  // Full snapshot.
            outcome_pool_state: crate::state::OutcomePool {
                market: outcome_pool.market.to_string(),
                outcome_id: outcome_pool.outcome_id,
                pool_amount: outcome_pool.pool_amount,
            },
            position_state: ctx.accounts.position.to_parity(), // Snapshot position.
        };

        // Run parity logic.
        let (new_market, new_pool, new_pos, _new_vault_amount, evt) =
            instructions::place_bet::place_bet(input).map_err(PitStopAnchorError::from)?;

        // Funds move (CPI) happens only after deterministic preconditions pass.
        let cpi_accounts = TransferChecked {
            from: ctx.accounts.user_usdc.to_account_info(), // Source: user token account.
            mint: ctx.accounts.usdc_mint.to_account_info(), // Mint: USDC.
            to: ctx.accounts.vault.to_account_info(),       // Destination: market vault.
            authority: ctx.accounts.user.to_account_info(), // Authority: user signs.
        };
        let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
        transfer_checked(cpi_ctx, args.amount, ctx.accounts.usdc_mint.decimals)?; // Execute transfer.

        // Commit state updates.
        ctx.accounts.market.apply_parity(&new_market); // Update market totals/status.
        outcome_pool.pool_amount = new_pool.pool_amount; // Update pool amount.
        {
            // OutcomePool is held as AccountInfo in this context; serialize manually.
            let mut data_mut = ctx.accounts.outcome_pool.try_borrow_mut_data()?; // Borrow writable data.
            let mut dst: &mut [u8] = &mut data_mut; // Create mutable byte slice cursor.
            outcome_pool.try_serialize(&mut dst)?; // Serialize back into account data.
        }
        ctx.accounts.position.apply_parity(&new_pos); // Update position.

        // Emit bet placed event.
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
        // Build parity input from current market snapshot.
        let now_ts = clock_unix_timestamp()?; // Time.
        let market_state = ctx.accounts.market.to_parity(); // Snapshot.
        let input = instructions::lock_market::LockMarketInput {
            authority: ctx.accounts.authority.key().to_string(),
            config_authority: ctx.accounts.config.authority.to_string(),
            market: ctx.accounts.market.key().to_string(),
            market_status: market_state.status,
            now_ts,
            lock_timestamp: market_state.lock_timestamp,
            market_state,
        };

        // Run parity lock.
        let (new_market, evt) =
            instructions::lock_market::lock_market(input).map_err(PitStopAnchorError::from)?;

        ctx.accounts.market.apply_parity(&new_market); // Commit.

        // Emit locked event.
        emit!(anchor_events::MarketLocked {
            market: ctx.accounts.market.key(),
            timestamp: evt.timestamp,
        });

        Ok(())
    }

    pub fn resolve_market(ctx: Context<ResolveMarket>, args: ResolveMarketArgs) -> Result<()> {
        // Resolve is an oracle action that needs the winning outcome pool.
        let now_ts = clock_unix_timestamp()?; // Time.

        // Load the winning pool deterministically (OutcomeMismatch if wrong/missing).
        let winning_pool = load_outcome_pool_checked(
            &ctx.accounts.winning_outcome_pool,
            ctx.accounts.market.key(),
            args.winning_outcome_id,
        )?;

        // Build parity resolve input.
        let market_state = ctx.accounts.market.to_parity();
        let input = instructions::resolve_market::ResolveMarketInput {
            oracle: ctx.accounts.oracle.key().to_string(), // Caller oracle.
            config_oracle: ctx.accounts.config.oracle.to_string(), // Expected oracle.
            market: ctx.accounts.market.key().to_string(), // Market.
            market_state,                                  // Snapshot.
            winning_outcome_id: args.winning_outcome_id,   // Winner.
            payload_hash: args.payload_hash,               // External attestation hash.
            winning_outcome_pool_state: Some(crate::state::OutcomePool {
                market: winning_pool.market.to_string(),
                outcome_id: winning_pool.outcome_id,
                pool_amount: winning_pool.pool_amount,
            }),
            now_ts,
        };

        // Run parity resolve.
        let (new_market, evt) = instructions::resolve_market::resolve_market(input)
            .map_err(PitStopAnchorError::from)?;

        ctx.accounts.market.apply_parity(&new_market); // Commit.

        // Emit resolved event.
        emit!(anchor_events::MarketResolved {
            market: ctx.accounts.market.key(),
            winning_outcome: evt.winning_outcome,
            payload_hash: evt.payload_hash,
            resolution_timestamp: evt.resolution_timestamp,
        });

        Ok(())
    }

    pub fn void_market(ctx: Context<VoidMarket>, args: VoidMarketArgs) -> Result<()> {
        // Void is an oracle action, similar to resolve but without a winner.
        let now_ts = clock_unix_timestamp()?; // Time.
        let market_state = ctx.accounts.market.to_parity(); // Snapshot.
        let input = instructions::void_market::VoidMarketInput {
            oracle: ctx.accounts.oracle.key().to_string(),
            config_oracle: ctx.accounts.config.oracle.to_string(),
            market: ctx.accounts.market.key().to_string(),
            payload_hash: args.payload_hash,
            now_ts,
            market_state,
        };

        // Run parity void.
        let (new_market, evt) =
            instructions::void_market::void_market(input).map_err(PitStopAnchorError::from)?;

        ctx.accounts.market.apply_parity(&new_market); // Commit.

        // Emit voided event.
        emit!(anchor_events::MarketVoided {
            market: ctx.accounts.market.key(),
            payload_hash: evt.payload_hash,
            resolution_timestamp: evt.resolution_timestamp,
        });

        Ok(())
    }

    pub fn claim_resolved(ctx: Context<ClaimResolved>, args: ClaimResolvedArgs) -> Result<()> {
        // Anchor boundary checks for token program + mint + vault + user token account.
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

        // Load the outcome pool referenced by the claim.
        let outcome_pool = load_outcome_pool_checked(
            &ctx.accounts.outcome_pool,
            ctx.accounts.market.key(),
            args.outcome_id,
        )?;

        // Build parity claim input.
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

        // Run parity claim.
        let (new_pos, _new_vault_amount, _new_user_amount, evt) =
            instructions::claim_resolved::claim_resolved(input)
                .map_err(PitStopAnchorError::from)?;

        // If payout is positive, transfer from vault to user (market PDA signs).
        if evt.payout > 0 {
            let cpi_accounts = TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.usdc_mint.to_account_info(),
                to: ctx.accounts.user_usdc.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            };

            // Re-derive the market PDA signer seeds so the program can sign.
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

            // Execute the checked transfer.
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            );
            transfer_checked(cpi_ctx, evt.payout, ctx.accounts.usdc_mint.decimals)?;
        }

        // Commit updated position (claimed flag + payout).
        ctx.accounts.position.apply_parity(&new_pos);

        // Emit claimed event.
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
        // Same boundary checks as resolved claim.
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

        // Build parity claim-voided input.
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

        // Run parity claim-voided.
        let (new_pos, _new_user_amount, _new_vault_amount, evt) =
            instructions::claim_voided::claim_voided(input).map_err(PitStopAnchorError::from)?;

        // If payout is positive, transfer principal back (market PDA signs).
        if evt.payout > 0 {
            let cpi_accounts = TransferChecked {
                from: ctx.accounts.vault.to_account_info(),
                mint: ctx.accounts.usdc_mint.to_account_info(),
                to: ctx.accounts.user_usdc.to_account_info(),
                authority: ctx.accounts.market.to_account_info(),
            };

            // Re-derive signer seeds for the market PDA.
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

            // Execute transfer back to user.
            let cpi_ctx = CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                cpi_accounts,
                signer,
            );
            transfer_checked(cpi_ctx, evt.payout, ctx.accounts.usdc_mint.decimals)?;
        }

        // Commit updated position.
        ctx.accounts.position.apply_parity(&new_pos);

        // Emit claimed event.
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
        // Boundary checks: token program, mint wiring, treasury wiring, vault wiring.
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

        // Build parity sweep input.
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

        // Run parity sweep.
        let (
            new_market,
            _new_treasury_amount,
            swept_amount,
            _vault_closed,
            _vault_exists,
            _used_seeds,
            evt,
        ) = instructions::sweep_remaining::sweep_remaining(input)
            .map_err(PitStopAnchorError::from)?;

        // Derive market PDA signer seeds.
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

        // If there’s anything to sweep, transfer from vault to treasury.
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

        // Close the vault token account (reclaims rent to close_destination).
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
        close_account(close_ctx)?; // Execute close CPI.

        // Commit market state update (status, totals, etc.).
        ctx.accounts.market.apply_parity(&new_market);

        // Emit swept event.
        emit!(anchor_events::MarketSweptEvent {
            market: ctx.accounts.market.key(),
            amount: evt.amount,
            to_treasury: ctx.accounts.treasury.key(),
            timestamp: evt.timestamp,
        });

        Ok(())
    }

    pub fn cancel_market(ctx: Context<CancelMarket>) -> Result<()> {
        // Boundary checks: pinned token program + vault matches market.
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

        // Build parity cancel input.
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

        // Run parity cancel.
        let (new_market, evt) =
            instructions::cancel_market::cancel_market(input).map_err(PitStopAnchorError::from)?;

        // Derive market PDA signer seeds.
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

        // Close vault (refund rent to close_destination).
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

        // Commit market state update.
        ctx.accounts.market.apply_parity(&new_market);

        // Emit cancelled event.
        emit!(anchor_events::MarketCancelled {
            market: ctx.accounts.market.key(),
            timestamp: evt.timestamp,
        });

        Ok(())
    }
}
