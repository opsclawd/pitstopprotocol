const assert = require('assert');
const { invokeLockMarketOnProgram } = require('../harness/lock_market_adapter');

(async function run() {
  const nowTs = 1_800_000_100;
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    market: 'MarketA',
    marketStatus: 'Open',
    nowTs,
    lockTimestamp: 1_800_000_000,
    marketState: { status: 'Open' },
  };

  // LKM-HP-001
  const ok = await invokeLockMarketOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.status, 'Locked');
  assert.equal(ok.event.name, 'MarketLocked');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.timestamp, nowTs);

  // LKM-REJ-001..003
  const cases = [
    [{ authority: 'Other' }, 'Unauthorized'],
    [{ marketStatus: 'Locked' }, 'MarketNotOpen'],
    [{ nowTs: base.lockTimestamp - 1 }, 'TooEarlyToLock'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokeLockMarketOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined);
  }

  console.log('lock_market conformance tests ok');
})();
