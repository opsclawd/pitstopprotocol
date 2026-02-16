function validateVoidMarketInput(input) {
  // VDM-REJ-001: oracle signer must match config.oracle.
  if (input.oracle !== input.configOracle) return 'UnauthorizedOracle';

  // VDM-REJ-002/003: only Locked markets can transition to Voided.
  if (input.marketStatus !== 'Locked') return 'MarketNotLocked';

  return null;
}

function executeVoidMarket(input) {
  const err = validateVoidMarketInput(input);
  if (err) return { ok: false, error: err };

  const market = {
    ...input.marketState,
    status: 'Voided',
    resolvedOutcome: null,
    resolutionPayloadHash: input.payloadHash,
    resolutionTimestamp: input.nowTs,
  };

  const event = {
    name: 'MarketVoided',
    market: input.market,
    payload_hash: input.payloadHash,
    resolution_timestamp: input.nowTs,
  };

  return { ok: true, market, event };
}

module.exports = { validateVoidMarketInput, executeVoidMarket };
