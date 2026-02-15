const assert = require('assert');
const constants = require('../../specs/constants.json');
const { validateInitializeInput } = require('../../packages/core/src/initialize_instruction.cjs');

(function run() {
  const base = {
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    usdcDecimals: 6,
    usdcMint: 'MintA',
    treasuryMint: 'MintA',
    treasuryOwner: 'TreasuryOwnerA',
    treasuryAuthority: 'TreasuryOwnerA',
    maxTotal: 1_000_000,
    maxPerUser: 100_000,
    claimWindowSecs: 3600,
  };

  assert.equal(validateInitializeInput(base), null);
  assert.equal(validateInitializeInput({ ...base, tokenProgram: 'TokenzFake' }), 'InvalidTokenProgram');
  assert.equal(validateInitializeInput({ ...base, usdcDecimals: 9 }), 'InvalidMintDecimals');
  assert.equal(validateInitializeInput({ ...base, treasuryMint: 'MintB' }), 'InvalidTreasuryMint');
  assert.equal(validateInitializeInput({ ...base, treasuryOwner: 'OtherOwner' }), 'InvalidTreasuryOwner');
  assert.equal(validateInitializeInput({ ...base, maxPerUser: 2_000_000 }), 'InvalidCap');
  assert.equal(validateInitializeInput({ ...base, maxTotal: 0 }), 'InvalidCap');
  assert.equal(validateInitializeInput({ ...base, claimWindowSecs: 0 }), 'InvalidClaimWindow');
  assert.equal(validateInitializeInput({ ...base, claimWindowSecs: constants.MAX_CLAIM_WINDOW_SECS + 1 }), 'InvalidClaimWindow');

  console.log('initialize spec tests ok');
})();
