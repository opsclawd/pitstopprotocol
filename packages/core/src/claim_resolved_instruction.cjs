const constants = require('../../../specs/constants.json');
const { computePrizePool, computePayout } = require('./protocol_primitives.cjs');

function validateClaimResolvedInput(input) {
  // Missing position PDA account -> framework account resolution failure (expected)
  if (input.positionExists === false) throw new Error('FrameworkAccountNotFound');

  // Failure ordering (locked): status gate is evaluated before vault/outcome usage.
  if (input.marketStatus !== 'Resolved') return 'MarketNotResolved';

  if (input.positionClaimed) return 'AlreadyClaimed';

  const claimEnd = input.resolutionTimestamp + input.claimWindowSecs;
  if (input.nowTs > claimEnd) return 'ClaimWindowExpired';

  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';

  // Deterministic wrapping: outcome pool missing/mismatched -> OutcomeMismatch
  if (
    !input.outcomePoolExists ||
    input.outcomePoolMarket !== input.market ||
    input.outcomePoolOutcomeId !== input.outcomeId
  ) {
    return 'OutcomeMismatch';
  }

  return null;
}

function executeClaimResolved(input) {
  const err = validateClaimResolvedInput(input);
  if (err) return { ok: false, error: err };

  const isWinner = input.outcomeId === input.winningOutcomeId;
  const prizePool = computePrizePool(input.marketTotalPool, input.feeBps);

  let payout = 0;
  if (isWinner) {
    try {
      payout = computePayout(input.positionAmount, prizePool, input.outcomePoolAmount);
    } catch (e) {
      if (e && e.message === 'DivisionByZero') return { ok: false, error: 'DivisionByZero' };
      return { ok: false, error: 'Overflow' };
    }

    if (payout > input.vaultAmount) return { ok: false, error: 'Underflow' };
  }

  const vaultAmount = input.vaultAmount - payout;
  const userUsdcAmount = input.userUsdcAmount + payout;

  const position = {
    ...input.positionState,
    claimed: true,
    payout,
  };

  const event = {
    name: 'Claimed',
    market: input.market,
    user: input.user,
    outcome_id: input.outcomeId,
    payout,
    claimed_at: input.nowTs,
  };

  return { ok: true, position, vaultAmount, userUsdcAmount, event };
}

module.exports = { validateClaimResolvedInput, executeClaimResolved };
