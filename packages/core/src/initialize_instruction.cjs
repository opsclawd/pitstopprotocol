const constants = require('../../../specs/constants.json');

function validateInitializeInput(input) {
  // INIT-REJ-001: token program must be pinned to SPL Token v1 at genesis.
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';
  // INIT-REJ-002: protocol is USDC-only for MVP (6 decimals).
  if (input.usdcDecimals !== constants.USDC_DECIMALS) return 'InvalidMintDecimals';
  // INIT-REJ-003: treasury token account must match configured USDC mint.
  if (input.treasuryMint !== input.usdcMint) return 'InvalidTreasuryMint';
  // INIT-REJ-004: treasury owner must match declared treasury authority.
  if (input.treasuryOwner !== input.treasuryAuthority) return 'InvalidTreasuryOwner';

  // INIT-REJ-005: caps must be positive and per-user cap cannot exceed market cap.
  if (
    !Number.isInteger(input.maxTotal) || input.maxTotal <= 0 ||
    !Number.isInteger(input.maxPerUser) || input.maxPerUser <= 0 ||
    input.maxPerUser > input.maxTotal
  ) return 'InvalidCap';

  // INIT-REJ-006: claim window is bounded [1, MAX_CLAIM_WINDOW_SECS].
  if (
    !Number.isInteger(input.claimWindowSecs) ||
    input.claimWindowSecs < 1 ||
    input.claimWindowSecs > constants.MAX_CLAIM_WINDOW_SECS
  ) return 'InvalidClaimWindow';

  return null;
}

function executeInitialize(input) {
  const err = validateInitializeInput(input);
  if (err) return { ok: false, error: err };

  const timestamp = input.nowTs;
  const config = {
    authority: input.authority,
    // MVP default: authority is the initial oracle until admin update flows exist.
    oracle: input.authority,
    usdcMint: input.usdcMint,
    treasury: input.treasury,
    treasuryAuthority: input.treasuryAuthority,
    // MVP default: initialize sets fee to 0 bps.
    feeBps: 0,
    paused: false,
    maxTotalPoolPerMarket: input.maxTotal,
    maxBetPerUserPerMarket: input.maxPerUser,
    claimWindowSecs: input.claimWindowSecs,
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
  };

  const event = {
    name: 'ConfigInitialized',
    authority: input.authority,
    // MVP default: authority is the initial oracle until admin update flows exist.
    oracle: input.authority,
    usdc_mint: input.usdcMint,
    treasury: input.treasury,
    fee_bps: 0,
    timestamp,
  };

  return { ok: true, config, event };
}

module.exports = { validateInitializeInput, executeInitialize };
