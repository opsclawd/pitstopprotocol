const assert = require('assert');
const constants = require('../../specs/constants.json');
const { validateClaimResolvedInput } = require('../../packages/core/src/claim_resolved_instruction.cjs');

(function run() {
  const base = {
    user: 'UserA',
    market: 'MarketA',
    outcomeId: 1,

    marketState: { status: 'Resolved', totalPool: 1_000_000 },
    nowTs: 1_800_000_100,
    resolutionTimestamp: 1_800_000_000,
    claimWindowSecs: 3600,

    feeBps: 200,

    positionExists: true,
    positionState: { amount: 100_000, claimed: false, payout: 0 },

    outcomePoolState: { market: 'MarketA', outcomeId: 1, poolAmount: 500_000 },

    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
  };

  assert.equal(validateClaimResolvedInput(base), null);
  assert.equal(validateClaimResolvedInput({ ...base, marketState: { ...base.marketState, status: 'Locked' } }), 'MarketNotResolved');
  assert.equal(validateClaimResolvedInput({ ...base, positionState: { ...base.positionState, claimed: true } }), 'AlreadyClaimed');
  assert.equal(
    validateClaimResolvedInput({ ...base, nowTs: base.resolutionTimestamp + base.claimWindowSecs + 1 }),
    'ClaimWindowExpired'
  );
  assert.equal(validateClaimResolvedInput({ ...base, outcomePoolState: null }), 'OutcomeMismatch');

  // Failure ordering (locked): status checked before outcome/vault usage.
  assert.equal(
    validateClaimResolvedInput({ ...base, marketState: { ...base.marketState, status: 'Swept' }, outcomePoolState: null }),
    'MarketNotResolved'
  );

  console.log('claim_resolved spec tests ok');
})();
