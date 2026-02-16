use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::state as parity_state;

/// Canonical protocol configuration PDA (`seeds = ["config"]`).
///
/// This mirrors the parity `state::Config` shape, but uses Anchor-friendly
/// account storage types (Pubkey/bool/u64/i64).
#[account]
#[derive(Debug)]
pub struct Config {
    pub authority: Pubkey,
    pub oracle: Pubkey,
    pub usdc_mint: Pubkey,
    pub treasury: Pubkey,
    pub treasury_authority: Pubkey,
    pub fee_bps: u16,
    pub paused: bool,
    pub max_total_pool_per_market: u64,
    pub max_bet_per_user_per_market: u64,
    pub claim_window_secs: i64,
    pub token_program: Pubkey,
}

impl Config {
    // discriminator (8) + fields
    pub const LEN: usize = 8
        + 32 // authority
        + 32 // oracle
        + 32 // usdc_mint
        + 32 // treasury
        + 32 // treasury_authority
        + 2 // fee_bps
        + 1 // paused
        + 8 // max_total_pool_per_market
        + 8 // max_bet_per_user_per_market
        + 8 // claim_window_secs
        + 32; // token_program
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum MarketStatus {
    Seeding,
    Open,
    Locked,
    Resolved,
    Voided,
    Swept,
}

/// Market account PDA (`seeds = ["market", market_id]`).
///
/// Stored as Anchor account state, converted to/from parity `state::Market`
/// inside handlers to preserve locked business semantics.
#[account]
#[derive(Debug)]
pub struct Market {
    pub market_id: [u8; 32],
    pub event_id: [u8; 32],
    pub lock_timestamp: i64,
    pub outcome_count: u8,
    pub max_outcomes: u8,
    pub total_pool: u64,
    pub status: MarketStatus,
    pub resolved_outcome: Option<u8>,
    pub resolution_payload_hash: [u8; 32],
    pub resolution_timestamp: i64,
    pub vault: Pubkey,
    pub market_type: u8,
    pub rules_version: u16,
}

impl Market {
    pub const LEN: usize = 8
        + 32 // market_id
        + 32 // event_id
        + 8 // lock_timestamp
        + 1 // outcome_count
        + 1 // max_outcomes
        + 8 // total_pool
        + 1 // status enum (anchor)
        + (1 + 1) // option<u8>
        + 32 // resolution_payload_hash
        + 8 // resolution_timestamp
        + 32 // vault
        + 1 // market_type
        + 2; // rules_version

    /// Anchor -> parity projection used before invoking pure instruction logic.
    pub fn to_parity(&self) -> parity_state::Market {
        parity_state::Market {
            market_id: self.market_id,
            event_id: self.event_id,
            lock_timestamp: self.lock_timestamp,
            outcome_count: self.outcome_count,
            max_outcomes: self.max_outcomes,
            total_pool: self.total_pool,
            status: match self.status {
                MarketStatus::Seeding => parity_state::MarketStatus::Seeding,
                MarketStatus::Open => parity_state::MarketStatus::Open,
                MarketStatus::Locked => parity_state::MarketStatus::Locked,
                MarketStatus::Resolved => parity_state::MarketStatus::Resolved,
                MarketStatus::Voided => parity_state::MarketStatus::Voided,
                MarketStatus::Swept => parity_state::MarketStatus::Swept,
            },
            resolved_outcome: self.resolved_outcome,
            resolution_payload_hash: self.resolution_payload_hash,
            resolution_timestamp: self.resolution_timestamp,
            vault: self.vault.to_string(),
            market_type: self.market_type,
            rules_version: self.rules_version,
        }
    }

    /// Parity -> Anchor commit used after successful validation/transition.
    pub fn apply_parity(&mut self, p: &parity_state::Market) {
        self.market_id = p.market_id;
        self.event_id = p.event_id;
        self.lock_timestamp = p.lock_timestamp;
        self.outcome_count = p.outcome_count;
        self.max_outcomes = p.max_outcomes;
        self.total_pool = p.total_pool;
        self.status = match p.status {
            parity_state::MarketStatus::Seeding => MarketStatus::Seeding,
            parity_state::MarketStatus::Open => MarketStatus::Open,
            parity_state::MarketStatus::Locked => MarketStatus::Locked,
            parity_state::MarketStatus::Resolved => MarketStatus::Resolved,
            parity_state::MarketStatus::Voided => MarketStatus::Voided,
            parity_state::MarketStatus::Swept => MarketStatus::Swept,
        };
        self.resolved_outcome = p.resolved_outcome;
        self.resolution_payload_hash = p.resolution_payload_hash;
        self.resolution_timestamp = p.resolution_timestamp;
        // vault Pubkey is set at create_market time and should not change.
        self.market_type = p.market_type;
        self.rules_version = p.rules_version;
    }
}

#[account]
#[derive(Debug)]
pub struct OutcomePool {
    pub market: Pubkey,
    pub outcome_id: u8,
    pub pool_amount: u64,
}

impl OutcomePool {
    pub const LEN: usize = 8 + 32 + 1 + 8;
}

/// User position for a given (market, user, outcome).
///
/// PDA: seeds = ["position", market, user, outcome_id]
#[account]
#[derive(Debug)]
pub struct Position {
    pub market: Pubkey,
    pub user: Pubkey,
    pub outcome_id: u8,
    pub amount: u64,
    pub claimed: bool,
    pub payout: u64,
}

impl Position {
    pub const LEN: usize = 8
        + 32 // market
        + 32 // user
        + 1 // outcome_id
        + 8 // amount
        + 1 // claimed
        + 8; // payout

    pub fn to_parity(&self) -> parity_state::Position {
        parity_state::Position {
            market: self.market.to_string(),
            user: self.user.to_string(),
            outcome_id: self.outcome_id,
            amount: self.amount,
            claimed: self.claimed,
            payout: self.payout,
        }
    }

    pub fn apply_parity(&mut self, p: &parity_state::Position) {
        self.outcome_id = p.outcome_id;
        self.amount = p.amount;
        self.claimed = p.claimed;
        self.payout = p.payout;
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct InitializeArgs {
    pub treasury_authority: Pubkey,
    pub max_total_pool_per_market: u64,
    pub max_bet_per_user_per_market: u64,
    pub claim_window_secs: i64,
}

/// Accounts for `initialize`.
///
/// Creates the single Config PDA and validates mint/treasury/token program
/// compatibility in handler logic.
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    #[account(
        init,
        payer = authority,
        space = Config::LEN,
        seeds = [b"config"],
        bump
    )]
    pub config: Account<'info, Config>,

    pub usdc_mint: InterfaceAccount<'info, Mint>,

    pub treasury: InterfaceAccount<'info, TokenAccount>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct CreateMarketArgs {
    pub market_id: [u8; 32],
    pub event_id: [u8; 32],
    pub lock_timestamp: i64,
    pub max_outcomes: u8,
    pub market_type: u8,
    pub rules_version: u16,
}

/// Accounts for `create_market`.
///
/// Creates the Market PDA and its vault ATA (owned by Market PDA) so later
/// betting/claim flows can move funds through a canonical escrow account.
#[derive(Accounts)]
#[instruction(args: CreateMarketArgs)]
pub struct CreateMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub config: Account<'info, Config>,

    #[account(
        init,
        payer = authority,
        space = Market::LEN,
        seeds = [b"market", args.market_id.as_ref()],
        bump
    )]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = authority,
        associated_token::mint = usdc_mint,
        associated_token::authority = market,
        associated_token::token_program = token_program
    )]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub usdc_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,

    pub associated_token_program: Program<'info, AssociatedToken>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AddOutcomeArgs {
    pub outcome_id: u8,
}

/// Accounts for `add_outcome`.
///
/// Creates one OutcomePool PDA per `(market, outcome_id)`.
#[derive(Accounts)]
#[instruction(args: AddOutcomeArgs)]
pub struct AddOutcome<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub config: Account<'info, Config>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    #[account(
        init,
        payer = authority,
        space = OutcomePool::LEN,
        seeds = [b"outcome", market.key().as_ref(), &[args.outcome_id]],
        bump
    )]
    pub outcome_pool: Account<'info, OutcomePool>,

    pub system_program: Program<'info, System>,
}

/// Accounts for `finalize_seeding`.
///
/// Mutates an existing Market account from Seeding -> Open after all outcomes
/// are present and lock timing allows transition.
#[derive(Accounts)]
pub struct FinalizeSeeding<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub config: Account<'info, Config>,

    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct PlaceBetArgs {
    pub outcome_id: u8,
    pub amount: u64,
}

/// Accounts for `place_bet`.
///
/// Transfers `amount` USDC from the user to the market vault and updates:
/// - market.total_pool
/// - outcome_pool.pool_amount
/// - position.amount
#[derive(Accounts)]
#[instruction(args: PlaceBetArgs)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    pub config: Account<'info, Config>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    /// CHECK: validated/decoded in handler so missing/wrong relation can map to OutcomeMismatch.
    #[account(mut)]
    pub outcome_pool: AccountInfo<'info>,

    #[account(
        init_if_needed,
        payer = user,
        space = Position::LEN,
        seeds = [b"position", market.key().as_ref(), user.key().as_ref(), &[args.outcome_id]],
        bump
    )]
    pub position: Account<'info, Position>,

    #[account(mut)]
    pub user_usdc: InterfaceAccount<'info, TokenAccount>,

    #[account(mut)]
    pub vault: InterfaceAccount<'info, TokenAccount>,

    pub usdc_mint: InterfaceAccount<'info, Mint>,

    pub token_program: Interface<'info, TokenInterface>,

    pub system_program: Program<'info, System>,
}

/// Accounts for `lock_market`.
#[derive(Accounts)]
pub struct LockMarket<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    pub config: Account<'info, Config>,

    #[account(mut)]
    pub market: Account<'info, Market>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct ResolveMarketArgs {
    pub winning_outcome_id: u8,
    pub payload_hash: [u8; 32],
}

/// Accounts for `resolve_market`.
#[derive(Accounts)]
#[instruction(args: ResolveMarketArgs)]
pub struct ResolveMarket<'info> {
    #[account(mut)]
    pub oracle: Signer<'info>,

    pub config: Account<'info, Config>,

    #[account(mut)]
    pub market: Account<'info, Market>,

    /// CHECK: validated/decoded in handler so missing/wrong relation can map to OutcomeMismatch.
    pub winning_outcome_pool: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct VoidMarketArgs {
    pub payload_hash: [u8; 32],
}

/// Accounts for `void_market`.
#[derive(Accounts)]
pub struct VoidMarket<'info> {
    #[account(mut)]
    pub oracle: Signer<'info>,

    pub config: Account<'info, Config>,

    #[account(mut)]
    pub market: Account<'info, Market>,
}
