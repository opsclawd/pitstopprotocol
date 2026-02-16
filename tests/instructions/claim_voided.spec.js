const assert = require('assert');
const { validateClaimVoidedInput } = require('../../packages/core/src/claim_voided_instruction.cjs');

(function run() {
  const base = {
    marketState: { status: 'Voided' },
    positionState: { claimed: false, payout: 0, amount: 250 },
    resolutionTimestamp: 1_800_000_000,
    claimWindowSecs: 3600,
    nowTs: 1_800_000_000,
  };

  assert.equal(validateClaimVoidedInput(base), null);
  assert.equal(validateClaimVoidedInput({ ...base, marketState: { status: 'Resolved' } }), 'MarketNotVoided');
  assert.equal(validateClaimVoidedInput({ ...base, positionState: { ...base.positionState, claimed: true } }), 'AlreadyClaimed');
  assert.equal(
    validateClaimVoidedInput({ ...base, nowTs: base.resolutionTimestamp + base.claimWindowSecs + 1 }),
    'ClaimWindowExpired'
  );

  // inclusive end boundary should pass
  assert.equal(
    validateClaimVoidedInput({ ...base, nowTs: base.resolutionTimestamp + base.claimWindowSecs }),
    null
  );

  console.log('claim_voided spec tests ok');
})();
