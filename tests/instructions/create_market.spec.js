const assert = require('assert');
const constants = require('../../specs/constants.json');
const { computeMarketIdHex } = require('../../packages/core/src/protocol_primitives.cjs');
const {
  MAX_OUTCOMES,
  validateCreateMarketInput,
} = require('../../packages/core/src/create_market_instruction.cjs');

(function run() {
  const eventIdHex = 'a'.repeat(64);
  const marketIdHex = computeMarketIdHex(eventIdHex, 0, 1);
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    marketIdHex,
    eventIdHex,
    lockTimestamp: 1_900_000_000,
    nowTs: 1_800_000_000,
    maxOutcomes: 20,
    marketType: 0,
    rulesVersion: 1,
  };

  assert.equal(validateCreateMarketInput(base), null);
  assert.equal(validateCreateMarketInput({ ...base, authority: 'Other' }), 'Unauthorized');
  assert.equal(validateCreateMarketInput({ ...base, tokenProgram: 'TokenzFake' }), 'InvalidTokenProgram');
  assert.equal(validateCreateMarketInput({ ...base, lockTimestamp: base.nowTs }), 'LockInPast');
  assert.equal(validateCreateMarketInput({ ...base, maxOutcomes: 0 }), 'ZeroOutcomes');
  assert.equal(validateCreateMarketInput({ ...base, maxOutcomes: MAX_OUTCOMES + 1 }), 'TooManyOutcomes');
  assert.equal(validateCreateMarketInput({ ...base, marketType: 2 }), 'UnsupportedMarketType');
  assert.equal(validateCreateMarketInput({ ...base, rulesVersion: 2 }), 'UnsupportedRulesVersion');
  assert.equal(validateCreateMarketInput({ ...base, marketIdHex: 'b'.repeat(64) }), 'InvalidMarketId');

  console.log('create_market spec tests ok');
})();
