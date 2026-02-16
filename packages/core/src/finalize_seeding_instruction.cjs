function validateFinalizeSeedingInput(input) {
  // FSE-REJ-001: only config authority can finalize seeding.
  if (input.authority !== input.configAuthority) return 'Unauthorized';
  // FSE-REJ-002: market must still be in Seeding.
  if (input.marketState.status !== 'Seeding') return 'MarketNotSeeding';
  // FSE-REJ-003: all outcomes must be seeded before open transition.
  if (input.marketState.outcomeCount !== input.marketState.maxOutcomes) return 'SeedingIncomplete';
  // FSE-REJ-004: cannot open at or after lock timestamp.
  if (input.nowTs >= input.marketState.lockTimestamp) return 'TooLateToOpen';
  return null;
}

function executeFinalizeSeeding(input) {
  const err = validateFinalizeSeedingInput(input);
  if (err) return { ok: false, error: err };

  const market = {
    ...input.marketState,
    status: 'Open',
  };

  const event = {
    name: 'MarketOpened',
    market: input.market,
    timestamp: input.nowTs,
  };

  return { ok: true, market, event };
}

module.exports = { validateFinalizeSeedingInput, executeFinalizeSeeding };
