const assert = require('assert');
const { invokeVoidMarketOnProgram } = require('../harness/void_market_adapter');

(async function run() {
  const nowTs = 1_800_000_100;
  const payloadHash = 'ab'.repeat(32);
  const base = {
    oracle: 'OracleA',
    configOracle: 'OracleA',
    market: 'MarketA',
    payloadHash,
    nowTs,
    marketState: {
      status: 'Locked',
      resolvedOutcome: 1,
      resolutionPayloadHash: '00'.repeat(32),
      resolutionTimestamp: 0,
    },
  };

  // VDM-HP-001
  const ok = await invokeVoidMarketOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.status, 'Voided');
  assert.equal(ok.market.resolvedOutcome, null);
  assert.equal(ok.market.resolutionPayloadHash, payloadHash);
  assert.equal(ok.market.resolutionTimestamp, nowTs);

  assert.equal(ok.event.name, 'MarketVoided');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.payload_hash, payloadHash);
  assert.equal(ok.event.resolution_timestamp, nowTs);

  // VDM-REJ-001..003
  const cases = [
    [{ oracle: 'Other' }, 'UnauthorizedOracle'],
    [{ marketState: { ...base.marketState, status: 'Open' } }, 'MarketNotLocked'],
    [{ marketState: { ...base.marketState, status: 'Resolved' } }, 'MarketNotLocked'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokeVoidMarketOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined);
  }

  console.log('void_market conformance tests ok');
})();
