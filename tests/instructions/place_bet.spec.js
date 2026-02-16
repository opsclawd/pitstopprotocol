const assert = require('assert');
const constants = require('../../specs/constants.json');
const { validatePlaceBetInput } = require('../../packages/core/src/place_bet_instruction.cjs');

(function run() {
  const base = {
    configPaused: false,
    nowTs: 1_800_000_000,
    outcomeId: 1,
    amount: 100,
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    market: 'MarketA',
    maxTotalPoolPerMarket: 10_000,
    maxBetPerUserPerMarket: 1000,
    marketState: { status: 'Open', lockTimestamp: 1_800_000_100, outcomeCount: 3, maxOutcomes: 3, totalPool: 1000 },
    outcomePoolState: { market: 'MarketA', outcomeId: 1, poolAmount: 400 },
    positionState: { amount: 200 },
  };

  assert.equal(validatePlaceBetInput(base), null);
  assert.equal(validatePlaceBetInput({ ...base, configPaused: true }), 'ProtocolPaused');
  assert.equal(validatePlaceBetInput({ ...base, marketState: { ...base.marketState, status: 'Locked' } }), 'MarketNotOpen');
  assert.equal(validatePlaceBetInput({ ...base, nowTs: base.marketState.lockTimestamp }), 'BettingClosed');
  assert.equal(validatePlaceBetInput({ ...base, outcomeId: 100 }), 'InvalidOutcomeId');
  assert.equal(
    validatePlaceBetInput({ ...base, marketState: { ...base.marketState, outcomeCount: 2 } }),
    'MarketNotReady'
  );
  assert.equal(validatePlaceBetInput({ ...base, amount: 0 }), 'ZeroAmount');
  assert.equal(
    validatePlaceBetInput({ ...base, marketState: { ...base.marketState, totalPool: 9_950 }, amount: 100 }),
    'MarketCapExceeded'
  );
  assert.equal(
    validatePlaceBetInput({ ...base, positionState: { ...base.positionState, amount: 950 }, amount: 100 }),
    'UserBetCapExceeded'
  );
  assert.equal(validatePlaceBetInput({ ...base, outcomePoolState: null }), 'OutcomeMismatch');
  assert.equal(validatePlaceBetInput({ ...base, tokenProgram: 'TokenzFake' }), 'InvalidTokenProgram');

  console.log('place_bet spec tests ok');
})();
