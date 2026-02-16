function validateResolveMarketInput(input) {
  // RSM-REJ-001: only oracle signer can resolve markets.
  if (input.oracle !== input.configOracle) return 'UnauthorizedOracle';
  // RSM-REJ-002: market must be Locked before resolving.
  if (input.marketState.status !== 'Locked') return 'MarketNotLocked';
  // RSM-REJ-003: winning_outcome_id must be in range [0, 99].
  if (!Number.isInteger(input.winningOutcomeId) || input.winningOutcomeId < 0 || input.winningOutcomeId > 99) return 'InvalidOutcomeId';

  // RSM-REJ-004: winning outcome must exist (modeled by outcome_count bound).
  // Spec allows InvalidOutcomeId/OutcomeMismatch; we deterministically map to InvalidOutcomeId here.
  if (!Number.isInteger(input.marketState.outcomeCount) || input.winningOutcomeId >= input.marketState.outcomeCount) return 'InvalidOutcomeId';

  // RSM-REJ-004: winning_outcome_pool must be present and bind to (market, outcome_id).
  // Spec requires OutcomeMismatch even for missing/uninitialized PDA.
  if (
    !input.winningOutcomePoolState ||
    input.winningOutcomePoolState.market !== input.market ||
    input.winningOutcomePoolState.outcomeId !== input.winningOutcomeId
  ) {
    return 'OutcomeMismatch';
  }

  return null;
}

function executeResolveMarket(input) {
  const err = validateResolveMarketInput(input);
  if (err) return { ok: false, error: err };

  const market = {
    ...input.marketState,
    status: 'Resolved',
    resolvedOutcome: input.winningOutcomeId,
    resolutionPayloadHash: input.payloadHashHex,
    resolutionTimestamp: input.nowTs,
  };

  const event = {
    name: 'MarketResolved',
    market: input.market,
    winning_outcome: input.winningOutcomeId,
    payload_hash: input.payloadHashHex,
    resolution_timestamp: input.nowTs,
  };

  return { ok: true, market, event };
}

module.exports = { validateResolveMarketInput, executeResolveMarket };
