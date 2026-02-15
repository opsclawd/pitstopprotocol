# initialize
Status: LOCKED (v1.0.1)

## 1) Purpose
Create the singleton `Config` account and lock protocol-wide operational constraints used by all later instructions.

Authoritative constants source: `specs/constants.json`

## 2) Inputs
Args:
- `treasury_authority: Pubkey`
- `max_total_pool_per_market: u64` (base units, USDC 6dp)
- `max_bet_per_user_per_market: u64` (base units, USDC 6dp)
- `claim_window_secs: i64` (seconds)

Valid ranges:
- `max_total_pool_per_market > 0`
- `max_bet_per_user_per_market > 0`
- `max_bet_per_user_per_market <= max_total_pool_per_market`
- `1 <= claim_window_secs <= MAX_CLAIM_WINDOW_SECS` (7,776,000)

## 3) Accounts
- `authority: Signer` (initializer + config authority)
- `config: init PDA ["config"]`
- `usdc_mint: Mint`
- `treasury: TokenAccount`
- `token_program: Program<Token>`
- `system_program: Program<System>`

Account constraints:
- `token_program.key() == REQUIRED_TOKEN_PROGRAM` (SPL Token v1)
- `usdc_mint.decimals == 6`
- `treasury.mint == usdc_mint.key()`
- `treasury.owner == treasury_authority`

## 4) Preconditions (`require!` map)
- wrong token program -> `InvalidTokenProgram`
- wrong mint decimals -> `InvalidMintDecimals`
- treasury mint mismatch -> `InvalidTreasuryMint`
- treasury owner mismatch -> `InvalidTreasuryOwner`
- invalid caps -> `InvalidCap`
- invalid claim window -> `InvalidClaimWindow`

## 5) State transitions
- Config lifecycle: `Uninitialized -> Active`
- Market state: unchanged

## 6) Token effects
- No token transfer CPI.
- No mint/burn.

## 7) Events
Must emit:
- `ConfigInitialized { authority, oracle, usdc_mint, treasury, fee_bps, timestamp }`

Field values at emit:
- `authority = authority.key()`
- `oracle = authority.key()` for MVP default
- `usdc_mint = usdc_mint.key()`
- `treasury = treasury.key()`
- `fee_bps = 0`
- `timestamp = Clock::unix_timestamp`

## 8) Postconditions
After success, `config` must satisfy:
- `config.authority == authority.key()`
- `config.oracle == authority.key()` (MVP default)
- `config.usdc_mint == usdc_mint.key()`
- `config.treasury == treasury.key()`
- `config.treasury_authority == treasury_authority`
- `config.fee_bps == 0`
- `config.paused == false`
- `config.max_total_pool_per_market == arg.max_total_pool_per_market`
- `config.max_bet_per_user_per_market == arg.max_bet_per_user_per_market`
- `config.claim_window_secs == arg.claim_window_secs`
- `config.token_program == REQUIRED_TOKEN_PROGRAM`

## 9) Failure modes (condition -> error)
- token program != SPL Token v1 -> `InvalidTokenProgram`
- mint decimals != 6 -> `InvalidMintDecimals`
- treasury.mint != usdc_mint -> `InvalidTreasuryMint`
- treasury.owner != treasury_authority -> `InvalidTreasuryOwner`
- caps invalid -> `InvalidCap`
- claim window out of bounds -> `InvalidClaimWindow`

## 10) Security notes
- Pins token program at genesis to prevent Token-2022/swap injection.
- Validates treasury ownership at init so sweep destination can be trusted as configured.
- Makes authority/oracle trust assumptions explicit from the first instruction.

## 11) Test requirements (must exist before implementation)
- `INIT-HP-001` happy path creates config with expected fields + emits event
- `INIT-REJ-001` wrong token program rejected
- `INIT-REJ-002` mint decimals != 6 rejected
- `INIT-REJ-003` treasury mint mismatch rejected
- `INIT-REJ-004` treasury owner mismatch rejected
- `INIT-REJ-005` invalid caps rejected
- `INIT-REJ-006` invalid claim window rejected
