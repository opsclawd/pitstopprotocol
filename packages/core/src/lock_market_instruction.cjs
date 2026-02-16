function validateLockMarketInput(input) {
  if (input.authority !== input.configAuthority) return 'Unauthorized'; // LKM-REJ-001
  if (input.marketState.status !== 'Open') return 'MarketNotOpen'; // LKM-REJ-002
  if (input.nowTs < input.marketState.lockTimestamp) return 'TooEarlyToLock'; // LKM-REJ-003
  return null;
}

function executeLockMarket(input) {
  const err = validateLockMarketInput(input);
  if (err) return { ok: false, error: err };

  const market = { ...input.marketState, status: 'Locked' };
  const event = {
    name: 'MarketLocked',
    market: input.market,
    timestamp: input.nowTs,
  };

  return { ok: true, market, event };
}

module.exports = { validateLockMarketInput, executeLockMarket };
