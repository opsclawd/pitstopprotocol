const assert = require('assert');
const { validateCancelMarketInput } = require('../../packages/core/src/cancel_market_instruction.cjs');

(function run() {
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    closeDestination: 'AuthA',
    marketStatus: 'Seeding',
    nowTs: 1_799_999_999,
    lockTimestamp: 1_800_000_000,
    marketTotalPool: 0,
    vaultAmount: 0,
  };

  assert.equal(validateCancelMarketInput(base), null);

  // CNL-REJ-001..005
  assert.equal(validateCancelMarketInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateCancelMarketInput({ ...base, marketStatus: 'Open' }), 'MarketNotSeeding');
  assert.equal(validateCancelMarketInput({ ...base, nowTs: base.lockTimestamp }), 'TooLateToCancel');
  assert.equal(validateCancelMarketInput({ ...base, marketTotalPool: 1 }), 'MarketHasBets');
  assert.equal(validateCancelMarketInput({ ...base, vaultAmount: 1 }), 'VaultNotEmpty');

  // CNL-ADV-001
  assert.equal(validateCancelMarketInput({ ...base, closeDestination: 'Other' }), 'Unauthorized');

  console.log('cancel_market spec tests ok');
})();
