const assert = require('assert');
const constants = require('../../specs/constants.json');
const { computeMarketIdHex } = require('../../packages/core/src/protocol_primitives.cjs');
const { invokeCreateMarketOnProgram } = require('../harness/create_market_adapter');

(async function run() {
  const nowTs = 1_800_000_000;
  const eventIdHex = 'a'.repeat(64);
  const marketIdHex = computeMarketIdHex(eventIdHex, 0, 1);

  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    market: 'MarketPdaA',
    vault: 'VaultAtaA',
    marketIdHex,
    eventIdHex,
    lockTimestamp: nowTs + 3600,
    nowTs,
    maxOutcomes: 20,
    marketType: 0,
    rulesVersion: 1,
  };

  // CRM-HP-001
  const ok = await invokeCreateMarketOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.status, 'Seeding');
  assert.equal(ok.market.outcomeCount, 0);
  assert.equal(ok.market.totalPool, 0);
  assert.equal(ok.market.resolvedOutcome, null);
  assert.equal(ok.market.resolutionTimestamp, 0);
  assert.equal(ok.market.vault, base.vault);
  assert.equal(ok.event.name, 'MarketCreated');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.market_id, base.marketIdHex);

  // CRM-REJ-001..006 (+ split rows)
  const cases = [
    [{ authority: 'Other' }, 'Unauthorized'],
    [{ tokenProgram: 'TokenzFake' }, 'InvalidTokenProgram'],
    [{ lockTimestamp: nowTs }, 'LockInPast'],
    [{ maxOutcomes: 0 }, 'ZeroOutcomes'],
    [{ maxOutcomes: 101 }, 'TooManyOutcomes'],
    [{ marketType: 2 }, 'UnsupportedMarketType'],
    [{ rulesVersion: 2 }, 'UnsupportedRulesVersion'],
    [{ marketIdHex: 'b'.repeat(64) }, 'InvalidMarketId'],
  ];

  for (const [patch, expected] of cases) {
    const out = await invokeCreateMarketOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined, 'failed instruction must not emit event');
  }

  console.log('create_market conformance tests ok');
})();
