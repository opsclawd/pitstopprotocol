const assert = require('assert');
const { validateResolveMarketInput } = require('../../packages/core/src/resolve_market_instruction.cjs');

(function run() {
  const base = {
    oracle: 'OracleA',
    configOracle: 'OracleA',
    market: 'MarketA',
    marketStatus: 'Locked',
    marketOutcomeCount: 3,
    winningOutcomeId: 1,
    payloadHashHex: 'ab'.repeat(32),
    winningOutcomePoolExists: true,
    winningOutcomePoolMarket: 'MarketA',
    winningOutcomePoolOutcomeId: 1,
  };

  assert.equal(validateResolveMarketInput(base), null);
  assert.equal(validateResolveMarketInput({ ...base, oracle: 'Other' }), 'UnauthorizedOracle');
  assert.equal(validateResolveMarketInput({ ...base, marketStatus: 'Open' }), 'MarketNotLocked');
  assert.equal(validateResolveMarketInput({ ...base, winningOutcomeId: 100 }), 'InvalidOutcomeId');
  assert.equal(validateResolveMarketInput({ ...base, winningOutcomeId: 2, marketOutcomeCount: 2 }), 'InvalidOutcomeId');
  assert.equal(validateResolveMarketInput({ ...base, winningOutcomePoolExists: false }), 'OutcomeMismatch');

  console.log('resolve_market spec tests ok');
})();
