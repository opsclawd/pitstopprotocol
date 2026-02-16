function validateCancelMarketInput(input) {
  // CNL-REJ-001: authority must match config authority.
  if (input.authority !== input.configAuthority) return 'Unauthorized';

  // CNL-ADV-001: closeDestination expected to equal authority.
  if (input.closeDestination !== input.authority) return 'Unauthorized';

  // CNL-REJ-002: only Seeding markets can be cancelled.
  if (input.marketState.status !== 'Seeding') return 'MarketNotSeeding';

  // CNL-REJ-003: cancel allowed strictly before lockTimestamp.
  if (input.nowTs >= input.marketState.lockTimestamp) return 'TooLateToCancel';

  // CNL-REJ-004: market must have no bets (canonical source: market account state).
  if (input.marketState.totalPool !== 0) return 'MarketHasBets';

  // CNL-REJ-005: vault must be empty before close.
  if (input.vaultAmount !== 0) return 'VaultNotEmpty';

  return null;
}

function executeCancelMarket(input) {
  const err = validateCancelMarketInput(input);
  if (err) return { ok: false, error: err };

  const market = {
    ...input.marketState,
    status: 'Voided',
    totalPool: input.marketState.totalPool,
    resolvedOutcome: null,
    resolutionTimestamp: input.nowTs,
    resolutionPayloadHash: '0'.repeat(64),
  };

  const event = {
    name: 'MarketCancelled',
    market: input.market,
    timestamp: input.nowTs,
  };

  return { ok: true, market, event };
}

module.exports = { validateCancelMarketInput, executeCancelMarket };
