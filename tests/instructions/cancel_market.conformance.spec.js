const assert = require('assert');
const { invokeCancelMarketOnProgram } = require('../harness/cancel_market_adapter');

(async function run() {
  const nowTs = 1_799_999_999;
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    closeDestination: 'AuthA',
    market: 'MarketA',
    marketStatus: 'Seeding',
    nowTs,
    lockTimestamp: 1_800_000_000,
    vaultAmount: 0,
    marketState: {
      status: 'Seeding',
      totalPool: 0,
      resolvedOutcome: null,
      resolutionPayloadHash: 'f'.repeat(64),
      resolutionTimestamp: 123,
    },
  };

  // CNL-HP-001
  const ok = await invokeCancelMarketOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.status, 'Voided');
  assert.equal(ok.market.resolutionTimestamp, nowTs);
  assert.equal(ok.market.resolutionPayloadHash, '0'.repeat(64));
  assert.equal(ok.market.resolvedOutcome, null);
  assert.equal(ok.event.name, 'MarketCancelled');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.timestamp, nowTs);

  // CNL-REJ-001..005
  const cases = [
    [{ authority: 'Other' }, 'Unauthorized'],
    [{ marketStatus: 'Open' }, 'MarketNotSeeding'],
    [{ nowTs: base.lockTimestamp }, 'TooLateToCancel'],
    [{ vaultAmount: 1 }, 'VaultNotEmpty'],
    [{ marketState: { ...base.marketState, totalPool: 1 } }, 'MarketHasBets'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokeCancelMarketOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined);
  }

  // mirrored marketTotalPool must not override canonical marketState.totalPool
  const mirrored = await invokeCancelMarketOnProgram({ ...base, marketTotalPool: 999 });
  assert.equal(mirrored.ok, true);

  // CNL-ADV-001
  const adv = await invokeCancelMarketOnProgram({ ...base, closeDestination: 'Other' });
  assert.equal(adv.ok, false);
  assert.equal(adv.error, 'Unauthorized');

  console.log('cancel_market conformance tests ok');
})();
