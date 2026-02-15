use crate::{
    constants::{MAX_CLAIM_WINDOW_SECS, REQUIRED_TOKEN_PROGRAM, USDC_DECIMALS},
    error::PitStopError,
    events::ConfigInitialized,
    state::Config,
};

#[derive(Debug, Clone)]
pub struct InitializeInput {
    pub authority: String,
    pub treasury_authority: String,
    pub usdc_mint: String,
    pub treasury: String,
    pub token_program: String,
    pub usdc_decimals: u8,
    pub treasury_mint: String,
    pub treasury_owner: String,
    pub max_total_pool_per_market: u64,
    pub max_bet_per_user_per_market: u64,
    pub claim_window_secs: i64,
    pub now_ts: i64,
}

pub fn initialize(input: InitializeInput) -> Result<(Config, ConfigInitialized), PitStopError> {
    if input.token_program != REQUIRED_TOKEN_PROGRAM {
        return Err(PitStopError::InvalidTokenProgram);
    }
    if input.usdc_decimals != USDC_DECIMALS {
        return Err(PitStopError::InvalidMintDecimals);
    }
    if input.treasury_mint != input.usdc_mint {
        return Err(PitStopError::InvalidTreasuryMint);
    }
    if input.treasury_owner != input.treasury_authority {
        return Err(PitStopError::InvalidTreasuryOwner);
    }
    if input.max_total_pool_per_market == 0
        || input.max_bet_per_user_per_market == 0
        || input.max_bet_per_user_per_market > input.max_total_pool_per_market
    {
        return Err(PitStopError::InvalidCap);
    }
    if input.claim_window_secs < 1 || input.claim_window_secs > MAX_CLAIM_WINDOW_SECS {
        return Err(PitStopError::InvalidClaimWindow);
    }

    let config = Config {
        authority: input.authority.clone(),
        oracle: input.authority.clone(),
        usdc_mint: input.usdc_mint.clone(),
        treasury: input.treasury.clone(),
        treasury_authority: input.treasury_authority.clone(),
        fee_bps: 0,
        paused: false,
        max_total_pool_per_market: input.max_total_pool_per_market,
        max_bet_per_user_per_market: input.max_bet_per_user_per_market,
        claim_window_secs: input.claim_window_secs,
        token_program: REQUIRED_TOKEN_PROGRAM.to_string(),
    };

    let evt = ConfigInitialized {
        authority: input.authority.clone(),
        oracle: input.authority,
        usdc_mint: input.usdc_mint,
        treasury: input.treasury,
        fee_bps: 0,
        timestamp: input.now_ts,
    };

    Ok((config, evt))
}
