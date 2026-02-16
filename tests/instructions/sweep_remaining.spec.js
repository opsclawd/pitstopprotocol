const assert = require('assert');
const constants = require('../../specs/constants.json');
const { validateSweepRemainingInput } = require('../../packages/core/src/sweep_remaining_instruction.cjs');

(function run() {
  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    marketStatus: 'Resolved',
    nowTs: 1_800_010_000,
    resolutionTimestamp: 1_800_000_000,
    claimWindowSecs: 5000,
    vaultAmount: 123,
    treasuryAmount: 1000,
    treasury: 'TreasuryA',
    treasuryMint: 'MintA',
    usdcMint: 'MintA',
    treasuryOwner: 'TreasuryAuthA',
    treasuryAuthority: 'TreasuryAuthA',
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    marketState: { status: 'Resolved' },
    market: 'MarketA',
  };

  // SWP-HP-001
  assert.equal(validateSweepRemainingInput(base), null);

  // SWP-AUTH-001
  assert.equal(validateSweepRemainingInput({ ...base, authority: 'Other' }), 'Unauthorized');

  // SWP-REJ-002 / SWP-IDEM-001 (status gate includes Swept deterministically)
  assert.equal(validateSweepRemainingInput({ ...base, marketStatus: 'Open' }), 'MarketNotResolved');
  assert.equal(validateSweepRemainingInput({ ...base, marketStatus: 'Swept' }), 'MarketNotResolved');

  // SWP-WIN-001
  const claimEnd = base.resolutionTimestamp + base.claimWindowSecs;
  assert.equal(validateSweepRemainingInput({ ...base, nowTs: claimEnd }), 'ClaimWindowNotExpired');

  // SWP-REJ-004 treasury constraints
  assert.equal(validateSweepRemainingInput({ ...base, treasuryMint: 'OtherMint' }), 'InvalidTreasuryMint');
  assert.equal(validateSweepRemainingInput({ ...base, treasuryOwner: 'OtherOwner' }), 'InvalidTreasuryOwner');

  // SWP-ADV-001 token program mismatch
  assert.equal(validateSweepRemainingInput({ ...base, tokenProgram: 'TokenzFake' }), 'InvalidTokenProgram');

  console.log('sweep_remaining spec tests ok');
})();
