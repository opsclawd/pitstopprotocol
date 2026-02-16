const assert = require('assert');
const constants = require('../../specs/constants.json');
const { validatePlaceBetInput } = require('../../packages/core/src/place_bet_instruction.cjs');

(function run() {
  const base = {
    configPaused: false,
    marketStatus: 'Open',
    nowTs: 1_800_000_000,
    marketLockTimestamp: 1_800_000_100,
    outcomeId: 1,
    marketOutcomeCount: 3,
    marketMaxOutcomes: 3,
    amount: 100,
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    outcomePoolExists: true,
    outcomePoolMarket: 'MarketA',
    market: 'MarketA',
    outcomePoolOutcomeId: 1,
    marketTotalPool: 1000,
    maxTotalPoolPerMarket: 10_000,
    userPositionAmount: 200,
    maxBetPerUserPerMarket: 1000,
  };

  assert.equal(validatePlaceBetInput(base), null);
  assert.equal(validatePlaceBetInput({ ...base, configPaused: true }), 'ProtocolPaused');
  assert.equal(validatePlaceBetInput({ ...base, marketStatus: 'Locked' }), 'MarketNotOpen');
  assert.equal(validatePlaceBetInput({ ...base, nowTs: base.marketLockTimestamp }), 'BettingClosed');
  assert.equal(validatePlaceBetInput({ ...base, outcomeId: 100 }), 'InvalidOutcomeId');
  assert.equal(validatePlaceBetInput({ ...base, marketOutcomeCount: 2 }), 'MarketNotReady');
  assert.equal(validatePlaceBetInput({ ...base, amount: 0 }), 'ZeroAmount');
  assert.equal(validatePlaceBetInput({ ...base, marketTotalPool: 9_950, amount: 100 }), 'MarketCapExceeded');
  assert.equal(validatePlaceBetInput({ ...base, userPositionAmount: 950, amount: 100 }), 'UserBetCapExceeded');
  assert.equal(validatePlaceBetInput({ ...base, outcomePoolExists: false }), 'OutcomeMismatch');
  assert.equal(validatePlaceBetInput({ ...base, tokenProgram: 'TokenzFake' }), 'InvalidTokenProgram');

  console.log('place_bet spec tests ok');
})();
