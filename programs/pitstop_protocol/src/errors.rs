use anchor_lang::prelude::*;

#[error_code]
pub enum PitStopError {
    #[msg("Invalid driver count. Must be between 2 and MAX_DRIVERS.")]
    InvalidDriverCount,
    #[msg("Invalid market times. close_ts must be greater than open_ts.")]
    InvalidMarketTimes,
    #[msg("Invalid fee_bps. Must be <= BPS_DENOMINATOR.")]
    InvalidFeeBps,

    #[msg("Market is not open for betting.")]
    MarketNotOpen,
    #[msg("Betting has not started for this market.")]
    BettingNotStarted,
    #[msg("Betting is closed for this market.")]
    BettingClosed,
    #[msg("Invalid driver index for this market.")]
    InvalidDriverIndex,
    #[msg("Position is locked to a different driver index.")]
    DriverSelectionLocked,
    #[msg("Bet amount is below minimum.")]
    BetTooSmall,
    #[msg("Math overflow while updating totals.")]
    MathOverflow,
    #[msg("Position account invariants failed.")]
    PositionInvariantViolation,
    #[msg("Invalid market configuration.")]
    InvalidMarketConfig,

}
