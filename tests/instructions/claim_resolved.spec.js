const assert = require('assert');
const constants = require('../../specs/constants.json');
const { validateClaimResolvedInput } = require('../../packages/core/src/claim_resolved_instruction.cjs');

(function run() {
  const base = {
    user: 'UserA',
    market: 'MarketA',
    outcomeId: 1,

    marketStatus: 'Resolved',
    nowTs: 1_800_000_100,
    resolutionTimestamp: 1_800_000_000,
    claimWindowSecs: 3600,

    feeBps: 200,
    marketTotalPool: 1_000_000,

    positionExists: true,
    positionClaimed: false,

    outcomePoolExists: true,
    outcomePoolMarket: 'MarketA',
    outcomePoolOutcomeId: 1,

    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
  };

  assert.equal(validateClaimResolvedInput(base), null);
  assert.equal(validateClaimResolvedInput({ ...base, marketStatus: 'Locked' }), 'MarketNotResolved');
  assert.equal(validateClaimResolvedInput({ ...base, positionClaimed: true }), 'AlreadyClaimed');
  assert.equal(
    validateClaimResolvedInput({ ...base, nowTs: base.resolutionTimestamp + base.claimWindowSecs + 1 }),
    'ClaimWindowExpired'
  );
  assert.equal(validateClaimResolvedInput({ ...base, outcomePoolExists: false }), 'OutcomeMismatch');

  // Failure ordering (locked): status checked before outcome/vault usage.
  assert.equal(
    validateClaimResolvedInput({ ...base, marketStatus: 'Swept', outcomePoolExists: false }),
    'MarketNotResolved'
  );

  console.log('claim_resolved spec tests ok');
})();
