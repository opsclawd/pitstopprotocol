const assert = require('assert');
const { validateFinalizeSeedingInput } = require('../../packages/core/src/finalize_seeding_instruction.cjs');

(function run() {
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    marketStatus: 'Seeding',
    marketOutcomeCount: 3,
    marketMaxOutcomes: 3,
    nowTs: 1_800_000_000,
    lockTimestamp: 1_800_000_100,
  };

  assert.equal(validateFinalizeSeedingInput(base), null);
  assert.equal(validateFinalizeSeedingInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateFinalizeSeedingInput({ ...base, marketStatus: 'Open' }), 'MarketNotSeeding');
  assert.equal(validateFinalizeSeedingInput({ ...base, marketOutcomeCount: 2 }), 'SeedingIncomplete');
  assert.equal(validateFinalizeSeedingInput({ ...base, nowTs: base.lockTimestamp }), 'TooLateToOpen');

  console.log('finalize_seeding spec tests ok');
})();
