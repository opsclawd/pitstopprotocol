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

fn validate_initialize_preconditions(input: &InitializeInput) -> Result<(), PitStopError> {
    // INIT-REJ-001
    if input.token_program != REQUIRED_TOKEN_PROGRAM {
        return Err(PitStopError::InvalidTokenProgram);
    }
    // INIT-REJ-002
    if input.usdc_decimals != USDC_DECIMALS {
        return Err(PitStopError::InvalidMintDecimals);
    }
    // INIT-REJ-003
    if input.treasury_mint != input.usdc_mint {
        return Err(PitStopError::InvalidTreasuryMint);
    }
    // INIT-REJ-004
    if input.treasury_owner != input.treasury_authority {
        return Err(PitStopError::InvalidTreasuryOwner);
    }
    // INIT-REJ-005
    if input.max_total_pool_per_market == 0
        || input.max_bet_per_user_per_market == 0
        || input.max_bet_per_user_per_market > input.max_total_pool_per_market
    {
        return Err(PitStopError::InvalidCap);
    }
    // INIT-REJ-006
    if input.claim_window_secs < 1 || input.claim_window_secs > MAX_CLAIM_WINDOW_SECS {
        return Err(PitStopError::InvalidClaimWindow);
    }

    Ok(())
}

pub fn initialize(input: InitializeInput) -> Result<(Config, ConfigInitialized), PitStopError> {
    validate_initialize_preconditions(&input)?;

    let config = Config {
        authority: input.authority.clone(),
        // MVP default: authority is oracle until oracle-admin update instructions are added.
        oracle: input.authority.clone(),
        usdc_mint: input.usdc_mint.clone(),
        treasury: input.treasury.clone(),
        treasury_authority: input.treasury_authority.clone(),
        // MVP default fee at protocol genesis.
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

#[cfg(test)]
mod tests {
    use super::*;

    fn base_input() -> InitializeInput {
        InitializeInput {
            authority: "AuthA".to_string(),
            treasury_authority: "TreasuryOwnerA".to_string(),
            usdc_mint: "MintA".to_string(),
            treasury: "TreasuryA".to_string(),
            token_program: REQUIRED_TOKEN_PROGRAM.to_string(),
            usdc_decimals: USDC_DECIMALS,
            treasury_mint: "MintA".to_string(),
            treasury_owner: "TreasuryOwnerA".to_string(),
            max_total_pool_per_market: 1_000_000,
            max_bet_per_user_per_market: 100_000,
            claim_window_secs: 3600,
            now_ts: 1_800_000_000,
        }
    }

    #[test]
    fn init_hp_001_builds_config_and_event() {
        let out = initialize(base_input()).expect("initialize should pass");
        let (cfg, evt) = out;

        assert_eq!(cfg.authority, "AuthA");
        assert_eq!(cfg.oracle, "AuthA");
        assert_eq!(cfg.usdc_mint, "MintA");
        assert_eq!(cfg.treasury, "TreasuryA");
        assert_eq!(cfg.treasury_authority, "TreasuryOwnerA");
        assert_eq!(cfg.fee_bps, 0);
        assert!(!cfg.paused);
        assert_eq!(cfg.max_total_pool_per_market, 1_000_000);
        assert_eq!(cfg.max_bet_per_user_per_market, 100_000);
        assert_eq!(cfg.claim_window_secs, 3600);
        assert_eq!(cfg.token_program, REQUIRED_TOKEN_PROGRAM);

        assert_eq!(evt.authority, "AuthA");
        assert_eq!(evt.oracle, "AuthA");
        assert_eq!(evt.usdc_mint, "MintA");
        assert_eq!(evt.treasury, "TreasuryA");
        assert_eq!(evt.fee_bps, 0);
        assert_eq!(evt.timestamp, 1_800_000_000);
    }

    #[test]
    fn init_rej_001_to_006_error_mapping() {
        let mut bad = base_input();
        bad.token_program = "TokenzFake".to_string();
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidTokenProgram);

        let mut bad = base_input();
        bad.usdc_decimals = 9;
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidMintDecimals);

        let mut bad = base_input();
        bad.treasury_mint = "MintB".to_string();
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidTreasuryMint);

        let mut bad = base_input();
        bad.treasury_owner = "Other".to_string();
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidTreasuryOwner);

        let mut bad = base_input();
        bad.max_total_pool_per_market = 0;
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidCap);

        let mut bad = base_input();
        bad.max_bet_per_user_per_market = 0;
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidCap);

        let mut bad = base_input();
        bad.max_bet_per_user_per_market = 2_000_000;
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidCap);

        let mut bad = base_input();
        bad.claim_window_secs = 0;
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidClaimWindow);

        let mut bad = base_input();
        bad.claim_window_secs = MAX_CLAIM_WINDOW_SECS + 1;
        assert_eq!(initialize(bad).unwrap_err(), PitStopError::InvalidClaimWindow);
    }

    #[test]
    fn init_claim_window_inclusive_bounds() {
        let mut low = base_input();
        low.claim_window_secs = 1;
        assert!(initialize(low).is_ok());

        let mut high = base_input();
        high.claim_window_secs = MAX_CLAIM_WINDOW_SECS;
        assert!(initialize(high).is_ok());
    }

}
