const assert = require('assert');
const { validateLockMarketInput } = require('../../packages/core/src/lock_market_instruction.cjs');

(function run() {
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    marketStatus: 'Open',
    nowTs: 1_800_000_100,
    lockTimestamp: 1_800_000_000,
  };

  assert.equal(validateLockMarketInput(base), null);
  assert.equal(validateLockMarketInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateLockMarketInput({ ...base, marketStatus: 'Locked' }), 'MarketNotOpen');
  assert.equal(validateLockMarketInput({ ...base, nowTs: base.lockTimestamp - 1 }), 'TooEarlyToLock');

  console.log('lock_market spec tests ok');
})();
