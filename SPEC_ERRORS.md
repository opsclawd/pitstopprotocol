# SPEC_ERRORS.md
Version: v1.0.0
Status: DRAFT

Maps protocol failures to stable error names/codes.

## Initial taxonomy (draft)
- Unauthorized
- UnauthorizedOracle
- InvalidTokenProgram
- InvalidMintDecimals
- InvalidTreasuryMint
- InvalidTreasuryOwner
- InvalidCap
- InvalidClaimWindow
- FeeTooHigh
- LockInPast
- TooEarlyToLock
- BettingClosed
- MarketNotSeeding
- MarketNotOpen
- MarketNotLocked
- MarketNotResolved
- MarketNotVoided
- MarketNotReady
- AlreadyClaimed
- InvalidOutcomeId
- OutcomeMismatch
- ZeroAmount
- TooManyOutcomes
- MaxOutcomesReached
- ZeroOutcomes
- MarketCapExceeded
- UserBetCapExceeded
- UnsupportedMarketType
- UnsupportedRulesVersion
- InvalidMarketId
- ClaimWindowExpired
- ClaimWindowNotExpired
- TooLateToCancel
- MarketHasBets
- VaultNotEmpty
- SeedingIncomplete
- TooLateToOpen
- Overflow
- Underflow
- DivisionByZero
