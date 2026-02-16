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

  // Winners are paid proportionally from the net prize pool.
  // Losers still transition to claimed, but receive payout=0.
  const isWinner = input.outcomeId === input.winningOutcomeId;

  // Prize pool is total pool minus protocol fee (as defined in locked math primitives).
  const prizePool = computePrizePool(input.marketTotalPool, input.feeBps);

  let payout = 0;
  if (isWinner) {
    // Winner payout: floor(position.amount * prizePool / winningOutcomePool.amount)
    // with deterministic error mapping for harness conformance.
    try {
      payout = computePayout(input.positionAmount, prizePool, input.outcomePoolAmount);
    } catch (e) {
      if (e && e.message === 'DivisionByZero') return { ok: false, error: 'DivisionByZero' };
      return { ok: false, error: 'Overflow' };
    }

    // Defensive arithmetic guard: modeled transfer cannot exceed vault balance.
    if (payout > input.vaultAmount) return { ok: false, error: 'Underflow' };
  }

  // Apply transfer effects (or no-op transfer for losers where payout=0).
  const vaultAmount = input.vaultAmount - payout;
  const userUsdcAmount = input.userUsdcAmount + payout;

  // Position is terminal after claim regardless of winner/loser branch.
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
