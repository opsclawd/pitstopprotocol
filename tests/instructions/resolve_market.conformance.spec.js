const assert = require('assert');
const { invokeResolveMarketOnProgram } = require('../harness/resolve_market_adapter');

(async function run() {
  const nowTs = 1_800_000_500;
  const base = {
    oracle: 'OracleA',
    configOracle: 'OracleA',
    market: 'MarketA',
    winningOutcomeId: 1,
    payloadHashHex: 'ab'.repeat(32),
    winningOutcomePoolState: { market: 'MarketA', outcomeId: 1, poolAmount: 0 },
    nowTs,
    marketState: {
      status: 'Locked',
      outcomeCount: 3,
      resolvedOutcome: null,
      resolutionPayloadHash: '0'.repeat(64),
      resolutionTimestamp: 0,
    },
  };

  // RSM-HP-001
  const ok = await invokeResolveMarketOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.status, 'Resolved');
  assert.equal(ok.market.resolvedOutcome, base.winningOutcomeId);
  assert.equal(ok.market.resolutionPayloadHash, base.payloadHashHex);
  assert.equal(ok.market.resolutionTimestamp, nowTs);

  assert.equal(ok.event.name, 'MarketResolved');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.winning_outcome, base.winningOutcomeId);
  assert.equal(ok.event.payload_hash, base.payloadHashHex);
  assert.equal(ok.event.resolution_timestamp, nowTs);

  // RSM-REJ-001..004
  const cases = [
    [{ oracle: 'Other' }, 'UnauthorizedOracle'],
    [{ marketState: { ...base.marketState, status: 'Open' } }, 'MarketNotLocked'],
    [{ winningOutcomeId: 100 }, 'InvalidOutcomeId'],
    // RSM-REJ-004: winning outcome must exist in seeded outcomes.
    [{ winningOutcomeId: base.marketState.outcomeCount }, 'InvalidOutcomeId'],
    // Wrong PDA passed (relation mismatch) -> OutcomeMismatch.
    [{ winningOutcomePoolState: { ...base.winningOutcomePoolState, outcomeId: 2 } }, 'OutcomeMismatch'],
    [{ winningOutcomePoolState: { ...base.winningOutcomePoolState, market: 'OtherMarket' } }, 'OutcomeMismatch'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokeResolveMarketOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined);
  }

  // RSM-ADV-001: missing/uninitialized winning_outcome_pool must deterministically map to OutcomeMismatch.
  const missing = await invokeResolveMarketOnProgram({ ...base, winningOutcomePoolState: null });
  assert.equal(missing.ok, false);
  assert.equal(missing.error, 'OutcomeMismatch');

  console.log('resolve_market conformance tests ok');
})();
