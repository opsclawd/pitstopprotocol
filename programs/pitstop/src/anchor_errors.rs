use anchor_lang::prelude::*;

use crate::error::PitStopError;

#[error_code]
pub enum PitStopAnchorError {
    #[msg("Unauthorized")]
    Unauthorized,
    #[msg("UnauthorizedOracle")]
    UnauthorizedOracle,
    #[msg("ProtocolPaused")]
    ProtocolPaused,

    #[msg("InvalidTokenProgram")]
    InvalidTokenProgram,
    #[msg("InvalidMintDecimals")]
    InvalidMintDecimals,
    #[msg("InvalidTreasuryMint")]
    InvalidTreasuryMint,
    #[msg("InvalidTreasuryOwner")]
    InvalidTreasuryOwner,
    #[msg("InvalidCap")]
    InvalidCap,
    #[msg("InvalidClaimWindow")]
    InvalidClaimWindow,

    #[msg("LockInPast")]
    LockInPast,
    #[msg("TooEarlyToLock")]
    TooEarlyToLock,
    #[msg("BettingClosed")]
    BettingClosed,
    #[msg("TooLateToOpen")]
    TooLateToOpen,
    #[msg("TooLateToCancel")]
    TooLateToCancel,

    #[msg("MarketNotSeeding")]
    MarketNotSeeding,
    #[msg("MarketNotOpen")]
    MarketNotOpen,
    #[msg("MarketNotLocked")]
    MarketNotLocked,
    #[msg("MarketNotResolved")]
    MarketNotResolved,
    #[msg("MarketNotVoided")]
    MarketNotVoided,
    #[msg("MarketNotReady")]
    MarketNotReady,

    #[msg("AlreadyClaimed")]
    AlreadyClaimed,
    #[msg("ClaimWindowExpired")]
    ClaimWindowExpired,
    #[msg("ClaimWindowNotExpired")]
    ClaimWindowNotExpired,

    #[msg("InvalidOutcomeId")]
    InvalidOutcomeId,
    #[msg("ZeroOutcomes")]
    ZeroOutcomes,
    #[msg("TooManyOutcomes")]
    TooManyOutcomes,
    #[msg("MaxOutcomesReached")]
    MaxOutcomesReached,
    #[msg("OutcomeMismatch")]
    OutcomeMismatch,
    #[msg("SeedingIncomplete")]
    SeedingIncomplete,

    #[msg("ZeroAmount")]
    ZeroAmount,
    #[msg("MarketCapExceeded")]
    MarketCapExceeded,
    #[msg("UserBetCapExceeded")]
    UserBetCapExceeded,

    #[msg("MarketHasBets")]
    MarketHasBets,
    #[msg("VaultNotEmpty")]
    VaultNotEmpty,

    #[msg("UnsupportedMarketType")]
    UnsupportedMarketType,
    #[msg("UnsupportedRulesVersion")]
    UnsupportedRulesVersion,
    #[msg("InvalidMarketId")]
    InvalidMarketId,

    #[msg("Overflow")]
    Overflow,
    #[msg("Underflow")]
    Underflow,
    #[msg("DivisionByZero")]
    DivisionByZero,
}

impl From<PitStopError> for PitStopAnchorError {
    fn from(e: PitStopError) -> Self {
        match e {
            PitStopError::Unauthorized => Self::Unauthorized,
            PitStopError::UnauthorizedOracle => Self::UnauthorizedOracle,
            PitStopError::ProtocolPaused => Self::ProtocolPaused,
            PitStopError::InvalidTokenProgram => Self::InvalidTokenProgram,
            PitStopError::InvalidMintDecimals => Self::InvalidMintDecimals,
            PitStopError::InvalidTreasuryMint => Self::InvalidTreasuryMint,
            PitStopError::InvalidTreasuryOwner => Self::InvalidTreasuryOwner,
            PitStopError::InvalidCap => Self::InvalidCap,
            PitStopError::InvalidClaimWindow => Self::InvalidClaimWindow,
            PitStopError::LockInPast => Self::LockInPast,
            PitStopError::TooEarlyToLock => Self::TooEarlyToLock,
            PitStopError::BettingClosed => Self::BettingClosed,
            PitStopError::TooLateToOpen => Self::TooLateToOpen,
            PitStopError::TooLateToCancel => Self::TooLateToCancel,
            PitStopError::MarketNotSeeding => Self::MarketNotSeeding,
            PitStopError::MarketNotOpen => Self::MarketNotOpen,
            PitStopError::MarketNotLocked => Self::MarketNotLocked,
            PitStopError::MarketNotResolved => Self::MarketNotResolved,
            PitStopError::MarketNotVoided => Self::MarketNotVoided,
            PitStopError::MarketNotReady => Self::MarketNotReady,
            PitStopError::AlreadyClaimed => Self::AlreadyClaimed,
            PitStopError::ClaimWindowExpired => Self::ClaimWindowExpired,
            PitStopError::ClaimWindowNotExpired => Self::ClaimWindowNotExpired,
            PitStopError::InvalidOutcomeId => Self::InvalidOutcomeId,
            PitStopError::ZeroOutcomes => Self::ZeroOutcomes,
            PitStopError::TooManyOutcomes => Self::TooManyOutcomes,
            PitStopError::MaxOutcomesReached => Self::MaxOutcomesReached,
            PitStopError::OutcomeMismatch => Self::OutcomeMismatch,
            PitStopError::SeedingIncomplete => Self::SeedingIncomplete,
            PitStopError::ZeroAmount => Self::ZeroAmount,
            PitStopError::MarketCapExceeded => Self::MarketCapExceeded,
            PitStopError::UserBetCapExceeded => Self::UserBetCapExceeded,
            PitStopError::MarketHasBets => Self::MarketHasBets,
            PitStopError::VaultNotEmpty => Self::VaultNotEmpty,
            PitStopError::UnsupportedMarketType => Self::UnsupportedMarketType,
            PitStopError::UnsupportedRulesVersion => Self::UnsupportedRulesVersion,
            PitStopError::InvalidMarketId => Self::InvalidMarketId,
            PitStopError::Overflow => Self::Overflow,
            PitStopError::Underflow => Self::Underflow,
            PitStopError::DivisionByZero => Self::DivisionByZero,
        }
    }
}
