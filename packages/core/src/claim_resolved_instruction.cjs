const constants = require('../../../specs/constants.json');
const { computePrizePool, computePayout } = require('./protocol_primitives.cjs');

function validateClaimResolvedInput(input) {
  // CLR-ADV-001: missing position PDA is expected to fail at account resolution layer.
  // Keep this throw-based behavior in JS model so harness can assert framework parity.
  if (input.positionExists === false) throw new Error('FrameworkAccountNotFound');

  // CLR-ORD-001: evaluate market lifecycle status before any vault/outcome math checks.
  // This guarantees deterministic error precedence when multiple inputs are invalid.
  if (input.marketState.status !== 'Resolved') return 'MarketNotResolved';

  // CLR-REJ-002: claim is single-use; once claimed, all future attempts must fail.
  if (input.positionState.claimed) return 'AlreadyClaimed';

  // CLR-REJ-003: claim window is inclusive at resolutionTimestamp + claimWindowSecs.
  // Expiry only begins strictly after the inclusive end boundary.
  const claimEnd = input.resolutionTimestamp + input.claimWindowSecs;
  if (input.nowTs > claimEnd) return 'ClaimWindowExpired';

  // CLR-ADV-002: transfers are locked to canonical SPL Token Program id.
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';

  // CLR-REJ-004: winning_outcome_pool relation must be valid for deterministic payout math.
  // Missing account or any market/outcome mismatch collapses to OutcomeMismatch.
  if (
    !input.outcomePoolState ||
    input.outcomePoolState.market !== input.market ||
    input.outcomePoolState.outcomeId !== input.outcomeId
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
  const prizePool = computePrizePool(input.marketState.totalPool, input.feeBps);

  let payout = 0;
  if (isWinner) {
    // Winner payout: floor(position.amount * prizePool / winningOutcomePool.amount)
    // with deterministic error mapping for harness conformance.
    try {
      payout = computePayout(input.positionState.amount, prizePool, input.outcomePoolState.poolAmount);
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
