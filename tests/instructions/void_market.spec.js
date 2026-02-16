const assert = require('assert');
const { validateVoidMarketInput } = require('../../packages/core/src/void_market_instruction.cjs');

(function run() {
  const base = {
    oracle: 'OracleA',
    configOracle: 'OracleA',
    marketState: { status: 'Locked' },
  };

  assert.equal(validateVoidMarketInput(base), null);
  assert.equal(validateVoidMarketInput({ ...base, oracle: 'Other' }), 'UnauthorizedOracle');
  assert.equal(validateVoidMarketInput({ ...base, marketState: { ...base.marketState, status: 'Open' } }), 'MarketNotLocked');
  assert.equal(validateVoidMarketInput({ ...base, marketState: { ...base.marketState, status: 'Resolved' } }), 'MarketNotLocked');

  console.log('void_market spec tests ok');
})();
