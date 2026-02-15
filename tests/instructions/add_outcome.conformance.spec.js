const assert = require('assert');
const { invokeAddOutcomeOnProgram } = require('../harness/add_outcome_adapter');

(async function run() {
  // deterministic adapter timestamp for reproducible conformance tests.
  const nowTs = 1_800_000_000;
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    market: 'MarketPdaA',
    marketStatus: 'Seeding',
    marketOutcomeCount: 1,
    marketMaxOutcomes: 3,
    outcomeId: 2,
    outcomePoolMarket: 'MarketPdaA',
    marketState: {
      status: 'Seeding',
      outcomeCount: 1,
      maxOutcomes: 3,
    },
    nowTs,
  };

  // ADO-HP-001
  const ok = await invokeAddOutcomeOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.outcomePool.market, base.market);
  assert.equal(ok.outcomePool.outcomeId, base.outcomeId);
  assert.equal(ok.outcomePool.poolAmount, 0);
  assert.equal(ok.market.outcomeCount, 2);
  assert.equal(ok.event.name, 'OutcomeAdded');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.outcome_id, base.outcomeId);
  assert.equal(ok.event.outcome_count, 2);
  assert.equal(ok.event.timestamp, nowTs);

  // ADO-REJ-001..005
  const cases = [
    [{ authority: 'Other' }, 'Unauthorized'],
    [{ marketStatus: 'Open' }, 'MarketNotSeeding'],
    [{ outcomeId: 100 }, 'InvalidOutcomeId'],
    [{ marketOutcomeCount: 3 }, 'MaxOutcomesReached'],
    [{ outcomePoolMarket: 'OtherMarket' }, 'OutcomeMismatch'],
  ];

  for (const [patch, expected] of cases) {
    const out = await invokeAddOutcomeOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined, 'failed instruction must not emit event');
  }

  console.log('add_outcome conformance tests ok');
})();
