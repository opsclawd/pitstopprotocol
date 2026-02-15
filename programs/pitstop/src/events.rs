#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigInitialized {
    pub authority: String,
    pub oracle: String,
    pub usdc_mint: String,
    pub treasury: String,
    pub fee_bps: u16,
    pub timestamp: i64,
}
