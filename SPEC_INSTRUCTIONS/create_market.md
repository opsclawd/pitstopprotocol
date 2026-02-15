# create_market
Version: v1.0.1
Status: LOCKED

## 1) Purpose
Create a new market PDA in `Seeding` state and initialize its USDC vault ATA.

## 2) Inputs
Args:
- `market_id: [u8;32]`
- `event_id: [u8;32]`
- `lock_timestamp: i64` (unix seconds)
- `max_outcomes: u8` (1..=MAX_OUTCOMES)
- `market_type: u8` (MVP supports Winner=0)
- `rules_version: u16` (MVP supports 1)

## 3) Accounts
- `authority: Signer`
- `config: Account<Config>` (authority must match config.authority)
- `market: init PDA ["market", market_id]`
- `vault: init ATA(mint=config.usdc_mint, authority=market PDA)
- `usdc_mint: Mint` (must equal config.usdc_mint)
- `token_program: Program<Token>` (must equal config.token_program)
- `associated_token_program`
- `system_program`

## 4) Preconditions
- authority is config.authority -> `Unauthorized`
- token program pinned -> `InvalidTokenProgram`
- `lock_timestamp <= now` -> `LockInPast`
- `1 <= max_outcomes <= MAX_OUTCOMES` -> `ZeroOutcomes`/`TooManyOutcomes`
- supported market_type/version -> `UnsupportedMarketType`/`UnsupportedRulesVersion`
- on-chain recomputed market_id must match provided -> `InvalidMarketId`

## 5) Effects
- Initialize `market` with:
  - status=`Seeding`
  - outcome_count=0
  - total_pool=0
  - resolved_outcome=None
  - resolution fields zeroed
  - vault pubkey recorded

## 6) Token effects
- No transfer.
- Creates vault ATA.

## 7) Events
- `MarketCreated` must emit.

## 8) Postconditions
- market is Seeding and ready for `add_outcome`.
- vault ATA exists and is owned by market PDA.

## 9) Failure table
- bad authority -> `Unauthorized`
- bad token program -> `InvalidTokenProgram`
- invalid lock ts -> `LockInPast`
- invalid outcomes -> `ZeroOutcomes`/`TooManyOutcomes`
- invalid type/version -> `UnsupportedMarketType`/`UnsupportedRulesVersion`
- market_id mismatch -> `InvalidMarketId`

## 10) Security notes
- On-chain market_id verification prevents off-chain canonicalization drift attacks.
- Vault authority set to market PDA centralizes custody in program logic.

## 11) Required tests
- CRM-HP-001, CRM-REJ-001..006


## Event contract link
- Event spec reference: `SPEC_EVENTS.md` -> `MarketCreated`.
