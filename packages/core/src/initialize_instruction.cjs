const constants = require('../../../specs/constants.json');

function validateInitializeInput(input) {
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';
  if (input.usdcDecimals !== constants.USDC_DECIMALS) return 'InvalidMintDecimals';
  if (input.treasuryMint !== input.usdcMint) return 'InvalidTreasuryMint';
  if (input.treasuryOwner !== input.treasuryAuthority) return 'InvalidTreasuryOwner';

  if (
    !Number.isInteger(input.maxTotal) || input.maxTotal <= 0 ||
    !Number.isInteger(input.maxPerUser) || input.maxPerUser <= 0 ||
    input.maxPerUser > input.maxTotal
  ) return 'InvalidCap';

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
    oracle: input.authority,
    usdcMint: input.usdcMint,
    treasury: input.treasury,
    treasuryAuthority: input.treasuryAuthority,
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
    oracle: input.authority,
    usdc_mint: input.usdcMint,
    treasury: input.treasury,
    fee_bps: 0,
    timestamp,
  };

  return { ok: true, config, event };
}

module.exports = { validateInitializeInput, executeInitialize };
