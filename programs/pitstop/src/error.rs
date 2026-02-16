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
    OutcomeMismatch,

    ZeroAmount,
    ZeroOutcomes,
    TooManyOutcomes,
    MaxOutcomesReached,
    SeedingIncomplete,
    TooLateToOpen,

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
