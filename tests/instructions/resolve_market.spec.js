const assert = require('assert');
const { validateResolveMarketInput } = require('../../packages/core/src/resolve_market_instruction.cjs');

(function run() {
  const base = {
    oracle: 'OracleA',
    configOracle: 'OracleA',
    market: 'MarketA',
    marketState: { status: 'Locked', outcomeCount: 3 },
    winningOutcomeId: 1,
    payloadHashHex: 'ab'.repeat(32),
    winningOutcomePoolState: { market: 'MarketA', outcomeId: 1 },
  };

  assert.equal(validateResolveMarketInput(base), null);
  assert.equal(validateResolveMarketInput({ ...base, oracle: 'Other' }), 'UnauthorizedOracle');
  assert.equal(validateResolveMarketInput({ ...base, marketState: { ...base.marketState, status: 'Open' } }), 'MarketNotLocked');
  assert.equal(validateResolveMarketInput({ ...base, winningOutcomeId: 100 }), 'InvalidOutcomeId');
  assert.equal(
    validateResolveMarketInput({ ...base, winningOutcomeId: 2, marketState: { ...base.marketState, outcomeCount: 2 } }),
    'InvalidOutcomeId'
  );
  assert.equal(validateResolveMarketInput({ ...base, winningOutcomePoolState: null }), 'OutcomeMismatch');

  console.log('resolve_market spec tests ok');
})();
