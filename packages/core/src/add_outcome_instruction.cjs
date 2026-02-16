function validateAddOutcomeInput(input) {
  // ADO-REJ-001: only config authority can add outcomes.
  if (input.authority !== input.configAuthority) return 'Unauthorized';
  // ADO-REJ-002: market must still be in Seeding lifecycle phase.
  if (input.marketState.status !== 'Seeding') return 'MarketNotSeeding';
  // ADO-REJ-003: outcome_id must be in range [0, 99].
  if (!Number.isInteger(input.outcomeId) || input.outcomeId < 0 || input.outcomeId > 99) return 'InvalidOutcomeId';
  // ADO-REJ-004: cannot exceed market.max_outcomes.
  if (input.marketState.outcomeCount >= input.marketState.maxOutcomes) return 'MaxOutcomesReached';
  // ADO-REJ-005: outcome pool account relation must bind to the same market.
  if (input.outcomePoolState.market !== input.market) return 'OutcomeMismatch';

  return null;
}

function executeAddOutcome(input) {
  const err = validateAddOutcomeInput(input);
  if (err) return { ok: false, error: err };

  const updatedMarket = {
    ...input.marketState,
    outcomeCount: input.marketState.outcomeCount + 1,
  };

  const outcomePool = {
    ...input.outcomePoolState,
    market: input.market,
    outcomeId: input.outcomeId,
    poolAmount: 0,
  };

  const event = {
    name: 'OutcomeAdded',
    market: input.market,
    outcome_id: input.outcomeId,
    outcome_count: updatedMarket.outcomeCount,
    timestamp: input.nowTs,
  };

  return { ok: true, market: updatedMarket, outcomePool, event };
}

module.exports = { validateAddOutcomeInput, executeAddOutcome };
