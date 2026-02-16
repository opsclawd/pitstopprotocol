const constants = require('../../../specs/constants.json');

function validateSweepRemainingInput(input) {
  // SWP-AUTH-001: authority must match config authority.
  if (input.authority !== input.configAuthority) return 'Unauthorized';

  // Token program pinned.
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';

  // SWP-REJ-002: market must be in {Resolved, Voided}.
  if (input.marketStatus !== 'Resolved' && input.marketStatus !== 'Voided') return 'MarketNotResolved';

  // SWP-WIN-001: claim window must be expired.
  const claimWindowEnd = input.resolutionTimestamp + input.claimWindowSecs;
  if (input.nowTs <= claimWindowEnd) return 'ClaimWindowNotExpired';

  // Treasury constraints (mint + owner) must match config.
  if (input.treasuryMint !== input.usdcMint) return 'InvalidTreasuryMint';
  if (input.treasuryOwner !== input.treasuryAuthority) return 'InvalidTreasuryOwner';

  return null;
}

function executeSweepRemaining(input) {
  const err = validateSweepRemainingInput(input);
  if (err) return { ok: false, error: err };

  const amount = input.vaultAmount;
  const treasuryAmount = input.treasuryAmount + amount;

  const market = { ...input.marketState, status: 'Swept' };

  const event = {
    name: 'MarketSweptEvent',
    market: input.market,
    amount,
    to_treasury: input.treasury,
    timestamp: input.nowTs,
  };

  return {
    ok: true,
    market,
    treasuryAmount,
    sweptAmount: amount,
    // Modeled vault semantics: full transfer then close using market PDA signer seeds.
    vaultClosed: true,
    vaultAccountExists: false,
    closeUsedMarketPdaSeeds: true,
    event,
  };
}

module.exports = { validateSweepRemainingInput, executeSweepRemaining };
