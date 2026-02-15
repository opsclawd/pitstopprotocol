const assert = require('assert');
const { invokeFinalizeSeedingOnProgram } = require('../harness/finalize_seeding_adapter');

(async function run() {
  const nowTs = 1_800_000_000;
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    market: 'MarketPdaA',
    marketStatus: 'Seeding',
    marketOutcomeCount: 3,
    marketMaxOutcomes: 3,
    lockTimestamp: nowTs + 100,
    nowTs,
    marketState: {
      status: 'Seeding',
      outcomeCount: 3,
      maxOutcomes: 3,
    },
  };

  // FSE-HP-001
  const ok = await invokeFinalizeSeedingOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.status, 'Open');
  assert.equal(ok.event.name, 'MarketOpened');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.timestamp, nowTs);

  // FSE-REJ-001..004
  const cases = [
    [{ authority: 'Other' }, 'Unauthorized'],
    [{ marketStatus: 'Open' }, 'MarketNotSeeding'],
    [{ marketOutcomeCount: 2 }, 'SeedingIncomplete'],
    [{ nowTs: base.lockTimestamp }, 'TooLateToOpen'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokeFinalizeSeedingOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined, 'failed instruction must not emit event');
  }

  console.log('finalize_seeding conformance tests ok');
})();
