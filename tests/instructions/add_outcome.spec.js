const assert = require('assert');
const { validateAddOutcomeInput } = require('../../packages/core/src/add_outcome_instruction.cjs');

(function run() {
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    market: 'MarketPdaA',
    outcomeId: 0,
    marketState: { status: 'Seeding', outcomeCount: 0, maxOutcomes: 3 },
    outcomePoolState: { market: 'MarketPdaA', outcomeId: 0, poolAmount: 0 },
  };

  assert.equal(validateAddOutcomeInput(base), null);
  assert.equal(validateAddOutcomeInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateAddOutcomeInput({ ...base, marketState: { ...base.marketState, status: 'Open' } }), 'MarketNotSeeding');
  assert.equal(validateAddOutcomeInput({ ...base, outcomeId: 100 }), 'InvalidOutcomeId');
  assert.equal(validateAddOutcomeInput({ ...base, outcomeId: -1 }), 'InvalidOutcomeId');
  assert.equal(validateAddOutcomeInput({ ...base, outcomeId: 1.5 }), 'InvalidOutcomeId');
  assert.equal(
    validateAddOutcomeInput({ ...base, marketState: { ...base.marketState, outcomeCount: 3 } }),
    'MaxOutcomesReached'
  );
  assert.equal(
    validateAddOutcomeInput({ ...base, outcomePoolState: { ...base.outcomePoolState, market: 'OtherMarket' } }),
    'OutcomeMismatch'
  );

  console.log('add_outcome spec tests ok');
})();
