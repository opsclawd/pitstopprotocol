use anchor_lang::prelude::*;

#[error_code]
pub enum PitStopError {
    #[msg("Invalid driver count. Must be between 2 and MAX_DRIVERS.")]
    InvalidDriverCount,
    #[msg("Invalid market times. close_ts must be greater than open_ts.")]
    InvalidMarketTimes,
    #[msg("Invalid fee_bps. Must be <= BPS_DENOMINATOR.")]
    InvalidFeeBps,
}
