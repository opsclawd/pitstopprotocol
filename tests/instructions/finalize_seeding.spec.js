const assert = require('assert');
const { validateFinalizeSeedingInput } = require('../../packages/core/src/finalize_seeding_instruction.cjs');

(function run() {
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    nowTs: 1_800_000_000,
    marketState: { status: 'Seeding', outcomeCount: 3, maxOutcomes: 3, lockTimestamp: 1_800_000_100 },
  };

  assert.equal(validateFinalizeSeedingInput(base), null);
  assert.equal(validateFinalizeSeedingInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateFinalizeSeedingInput({ ...base, marketState: { ...base.marketState, status: 'Open' } }), 'MarketNotSeeding');
  assert.equal(
    validateFinalizeSeedingInput({ ...base, marketState: { ...base.marketState, outcomeCount: 2 } }),
    'SeedingIncomplete'
  );
  assert.equal(validateFinalizeSeedingInput({ ...base, nowTs: base.marketState.lockTimestamp }), 'TooLateToOpen');

  console.log('finalize_seeding spec tests ok');
})();
