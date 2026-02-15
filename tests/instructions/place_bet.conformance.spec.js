const assert = require('assert');
const constants = require('../../specs/constants.json');
const { invokePlaceBetOnProgram } = require('../harness/place_bet_adapter');

(async function run() {
  const nowTs = 1_800_000_000;
  const base = {
    configPaused: false,
    marketStatus: 'Open',
    nowTs,
    marketLockTimestamp: nowTs + 100,
    outcomeId: 1,
    marketOutcomeCount: 3,
    marketMaxOutcomes: 3,
    amount: 100,
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    outcomePoolExists: true,
    outcomePoolMarket: 'MarketA',
    outcomePoolOutcomeId: 1,
    market: 'MarketA',
    user: 'UserA',
    marketTotalPool: 1000,
    maxTotalPoolPerMarket: 10_000,
    userPositionAmount: 200,
    maxBetPerUserPerMarket: 1000,
    outcomePoolAmount: 400,
    vaultAmount: 1000,
    marketState: { totalPool: 1000, status: 'Open' },
    outcomePoolState: { poolAmount: 400 },
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
  assert.equal(ok.event.market_total_pool, 1100);
  assert.equal(ok.event.outcome_pool_amount, 500);

  // PBT-REJ-001..010
  const cases = [
    [{ configPaused: true }, 'ProtocolPaused'],
    [{ marketStatus: 'Locked' }, 'MarketNotOpen'],
    [{ nowTs: base.marketLockTimestamp }, 'BettingClosed'],
    [{ outcomeId: 100 }, 'InvalidOutcomeId'],
    [{ marketOutcomeCount: 2 }, 'MarketNotReady'],
    [{ amount: 0 }, 'ZeroAmount'],
    [{ marketTotalPool: 9_950 }, 'MarketCapExceeded'],
    [{ userPositionAmount: 950 }, 'UserBetCapExceeded'],
    [{ outcomePoolExists: false }, 'OutcomeMismatch'],
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
    [{ outcomePoolMarket: 'OtherMarket' }, 'OutcomeMismatch'],
    [{ outcomePoolOutcomeId: 2 }, 'OutcomeMismatch'],
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
