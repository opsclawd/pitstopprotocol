#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub authority: String,
    pub oracle: String,
    pub usdc_mint: String,
    pub treasury: String,
    pub treasury_authority: String,
    pub fee_bps: u16,
    pub paused: bool,
    pub max_total_pool_per_market: u64,
    pub max_bet_per_user_per_market: u64,
    pub claim_window_secs: i64,
    pub token_program: String,
}
