#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PitStopError {
    Unauthorized,
    UnauthorizedOracle,
    ProtocolPaused,

    InvalidTokenProgram,
    InvalidMintDecimals,
    InvalidTreasuryMint,
    InvalidTreasuryOwner,
    InvalidCap,
    InvalidClaimWindow,

    LockInPast,
    TooEarlyToLock,
    BettingClosed,
    TooLateToOpen,
    TooLateToCancel,

    MarketNotSeeding,
    MarketNotOpen,
    MarketNotLocked,
    MarketNotResolved,
    MarketNotVoided,
    MarketNotReady,

    AlreadyClaimed,
    ClaimWindowExpired,
    ClaimWindowNotExpired,

    InvalidOutcomeId,
    ZeroOutcomes,
    TooManyOutcomes,
    MaxOutcomesReached,
    OutcomeMismatch,
    SeedingIncomplete,

    ZeroAmount,
    MarketCapExceeded,
    UserBetCapExceeded,

    MarketHasBets,
    VaultNotEmpty,

    UnsupportedMarketType,
    UnsupportedRulesVersion,
    InvalidMarketId,

    Overflow,
    Underflow,
    DivisionByZero,
}
