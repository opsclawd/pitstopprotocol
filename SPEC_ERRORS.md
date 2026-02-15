# SPEC_ERRORS.md
Version: v1.1.1
Status: LOCKED

Stable protocol error taxonomy and instruction mapping.

## Error set (locked)
- Unauthorized
- UnauthorizedOracle
- ProtocolPaused
- InvalidTokenProgram
- InvalidMintDecimals
- InvalidTreasuryMint
- InvalidTreasuryOwner
- InvalidCap
- InvalidClaimWindow
- FeeTooHigh (reserved until admin fee-update instruction is implemented)
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

## Instruction mapping (condition -> error)

### initialize
- token program mismatch -> InvalidTokenProgram
- mint decimals != 6 -> InvalidMintDecimals
- treasury mint mismatch -> InvalidTreasuryMint
- treasury owner mismatch -> InvalidTreasuryOwner
- cap config invalid -> InvalidCap
- claim window invalid -> InvalidClaimWindow

### create_market
- authority mismatch -> Unauthorized
- token program mismatch -> InvalidTokenProgram
- lock timestamp <= now -> LockInPast
- max_outcomes == 0 -> ZeroOutcomes
- max_outcomes > MAX_OUTCOMES -> TooManyOutcomes
- unsupported market_type -> UnsupportedMarketType
- unsupported rules_version -> UnsupportedRulesVersion
- on-chain market_id recompute mismatch -> InvalidMarketId

### add_outcome
- authority mismatch -> Unauthorized
- market not Seeding -> MarketNotSeeding
- outcome_id > 99 -> InvalidOutcomeId
- outcome_count >= max_outcomes -> MaxOutcomesReached
- market/outcome account relation mismatch -> OutcomeMismatch

### finalize_seeding
- authority mismatch -> Unauthorized
- market not Seeding -> MarketNotSeeding
- outcome_count != max_outcomes -> SeedingIncomplete
- now >= lock_timestamp -> TooLateToOpen

### place_bet
- protocol paused -> ProtocolPaused
- market not Open -> MarketNotOpen
- now >= lock_timestamp -> BettingClosed
- outcome_id > 99 -> InvalidOutcomeId
- outcome_count != max_outcomes -> MarketNotReady
- amount == 0 -> ZeroAmount
- market cap exceeded -> MarketCapExceeded
- user cap exceeded -> UserBetCapExceeded
- outcome_pool mismatched relation -> OutcomeMismatch
- outcome_pool missing/uninitialized -> framework account failure unless explicitly wrapped
- token program mismatch -> InvalidTokenProgram
- user/vault mint or owner mismatch -> framework account constraint failure unless explicitly wrapped to protocol errors
- checked math overflow -> Overflow

### lock_market
- authority mismatch -> Unauthorized
- market not Open -> MarketNotOpen
- now < lock_timestamp -> TooEarlyToLock

### resolve_market
- signer != config.oracle -> UnauthorizedOracle
- market not Locked -> MarketNotLocked
- outcome_id > 99 -> InvalidOutcomeId
- outcome pool mismatched relation -> OutcomeMismatch
- outcome pool missing/uninitialized -> framework account failure unless explicitly wrapped

### void_market
- signer != config.oracle -> UnauthorizedOracle
- market not Locked -> MarketNotLocked

### claim_resolved
- market not Resolved -> MarketNotResolved
- already claimed -> AlreadyClaimed
- now > claim window end -> ClaimWindowExpired
- outcome pool missing/mismatched -> OutcomeMismatch
- winner_pool == 0 -> DivisionByZero
- checked math overflow/underflow -> Overflow/Underflow

### claim_voided
- market not Voided -> MarketNotVoided
- already claimed -> AlreadyClaimed
- now > claim window end -> ClaimWindowExpired
- checked math overflow/underflow -> Overflow/Underflow

### sweep_remaining
- authority mismatch -> Unauthorized
- market not in {Resolved, Voided} -> MarketNotResolved (deterministic lifecycle rejection)
- now <= claim window end -> ClaimWindowNotExpired
- treasury mismatch -> InvalidTreasuryMint/InvalidTreasuryOwner

### cancel_market
- authority mismatch -> Unauthorized
- market not Seeding -> MarketNotSeeding
- now >= lock_timestamp -> TooLateToCancel
- market.total_pool > 0 -> MarketHasBets
- vault.amount != 0 -> VaultNotEmpty

## Framework-level account failures
The following may surface as Anchor/Solana framework account resolution failures (not protocol errors), unless explicitly wrapped:
- required PDA account missing
- account owner mismatch not mapped to protocol error
- account discriminator/type mismatch

Where deterministic protocol errors are required by instruction spec, implementation must map to the corresponding protocol error.


## Authority and precedence
- Per-instruction specs in `SPEC_INSTRUCTIONS/*.md` are authoritative for precondition logic.
- `SPEC_ERRORS.md` is the canonical error registry + summary mapping and must remain consistent with instruction specs.
