const assert = require('assert');
const constants = require('../../specs/constants.json');
const { invokePlaceBetOnProgram } = require('../harness/place_bet_adapter');

(async function run() {
  const nowTs = 1_800_000_000;
  const base = {
    configPaused: false,
    nowTs,
    outcomeId: 1,
    amount: 100,
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    market: 'MarketA',
    user: 'UserA',
    maxTotalPoolPerMarket: 10_000,
    maxBetPerUserPerMarket: 1000,
    vaultAmount: 1000,
    marketState: { totalPool: 1000, status: 'Open', lockTimestamp: nowTs + 100, outcomeCount: 3, maxOutcomes: 3 },
    outcomePoolState: { poolAmount: 400, market: 'MarketA', outcomeId: 1 },
    positionState: { amount: 200 },
  };

  // PBT-HP-001/002 + invariants
  const ok = await invokePlaceBetOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.totalPool, 1100);
  assert.equal(ok.outcomePool.poolAmount, 500);
  assert.equal(ok.position.amount, 300);
  assert.equal(ok.vaultAmount, 1100);
  assert.equal(ok.market.totalPool, ok.vaultAmount, 'PBT-INV-002 pre-resolution vault == market total');
  assert.equal(ok.event.name, 'BetPlaced');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.user, base.user);
  assert.equal(ok.event.outcome_id, base.outcomeId);
  assert.equal(ok.event.amount, base.amount);
  assert.equal(ok.event.timestamp, nowTs);
  assert.equal(ok.event.market_total_pool, 1100);
  assert.equal(ok.event.outcome_pool_amount, 500);

  // PBT-HP-002: init_if_needed style new position starts at zero and increments.
  const newPos = await invokePlaceBetOnProgram({
    ...base,
    user: 'UserB',
    positionState: { amount: 0 },
    amount: 75,
    outcomePoolState: { ...base.outcomePoolState, poolAmount: 200 },
    marketState: { ...base.marketState, totalPool: 500 },
    vaultAmount: 500,
  });
  assert.equal(newPos.ok, true);
  assert.equal(newPos.position.amount, 75);
  assert.equal(newPos.market.totalPool, 575);
  assert.equal(newPos.outcomePool.poolAmount, 275);

  // PBT-INV-001: sum(outcome pools) == market.total_pool (modeled with multi-pool snapshot).
  const otherOutcomePoolAmount = 300;
  const modeledSumPools = newPos.outcomePool.poolAmount + otherOutcomePoolAmount;
  assert.equal(modeledSumPools, newPos.market.totalPool, 'sum(outcome pools) must equal market.total_pool');

  // PBT-REJ-001..010
  const cases = [
    [{ configPaused: true }, 'ProtocolPaused'],
    [{ marketState: { ...base.marketState, status: 'Locked' } }, 'MarketNotOpen'],
    [{ nowTs: base.marketState.lockTimestamp }, 'BettingClosed'],
    [{ outcomeId: 100 }, 'InvalidOutcomeId'],
    [{ marketState: { ...base.marketState, outcomeCount: 2 } }, 'MarketNotReady'],
    [{ amount: 0 }, 'ZeroAmount'],
    [{ marketState: { ...base.marketState, totalPool: 9_950 } }, 'MarketCapExceeded'],
    [{ positionState: { ...base.positionState, amount: 950 } }, 'UserBetCapExceeded'],
    [{ outcomePoolState: null }, 'OutcomeMismatch'],
    [{ tokenProgram: 'TokenzFake' }, 'InvalidTokenProgram'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokePlaceBetOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined);
  }

  // PBT-ADV-001..004 basic adversarial mismatches
  const advCases = [
    [{ outcomePoolState: { ...base.outcomePoolState, market: 'OtherMarket' } }, 'OutcomeMismatch'],
    [{ outcomePoolState: { ...base.outcomePoolState, outcomeId: 2 } }, 'OutcomeMismatch'],
    [{ amount: -1 }, 'ZeroAmount'],
    [{ amount: 1.5 }, 'ZeroAmount'],
  ];
  for (const [patch, expected] of advCases) {
    const out = await invokePlaceBetOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
  }

  console.log('place_bet conformance tests ok');
})();
