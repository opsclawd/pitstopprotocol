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
    vaultAmount: 0,
    marketState: { totalPool: 0 },
  };

  assert.equal(validateCancelMarketInput(base), null);

  // CNL-REJ-001..005
  assert.equal(validateCancelMarketInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateCancelMarketInput({ ...base, marketStatus: 'Open' }), 'MarketNotSeeding');
  assert.equal(validateCancelMarketInput({ ...base, nowTs: base.lockTimestamp }), 'TooLateToCancel');
  assert.equal(validateCancelMarketInput({ ...base, vaultAmount: 1 }), 'VaultNotEmpty');
  assert.equal(validateCancelMarketInput({ ...base, marketState: { ...base.marketState, totalPool: 1 } }), 'MarketHasBets');
  // mirrored marketTotalPool is ignored; canonical marketState.totalPool governs
  assert.equal(validateCancelMarketInput({ ...base, marketTotalPool: 999, marketState: { ...base.marketState, totalPool: 0 } }), null);

  // CNL-ADV-001
  assert.equal(validateCancelMarketInput({ ...base, closeDestination: 'Other' }), 'Unauthorized');

  console.log('cancel_market spec tests ok');
})();
