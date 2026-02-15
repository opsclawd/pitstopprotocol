const assert = require('assert');
const { validateAddOutcomeInput } = require('../../packages/core/src/add_outcome_instruction.cjs');

(function run() {
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    market: 'MarketPdaA',
    marketStatus: 'Seeding',
    marketOutcomeCount: 0,
    marketMaxOutcomes: 3,
    outcomeId: 0,
    outcomePoolMarket: 'MarketPdaA',
  };

  assert.equal(validateAddOutcomeInput(base), null);
  assert.equal(validateAddOutcomeInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateAddOutcomeInput({ ...base, marketStatus: 'Open' }), 'MarketNotSeeding');
  assert.equal(validateAddOutcomeInput({ ...base, outcomeId: 100 }), 'InvalidOutcomeId');
  assert.equal(validateAddOutcomeInput({ ...base, marketOutcomeCount: 3 }), 'MaxOutcomesReached');
  assert.equal(validateAddOutcomeInput({ ...base, outcomePoolMarket: 'OtherMarket' }), 'OutcomeMismatch');

  console.log('add_outcome spec tests ok');
})();
