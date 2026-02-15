const assert = require('assert');

const REQUIRED_TOKEN_PROGRAM = 'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA';
const MAX_CLAIM_WINDOW_SECS = 90 * 24 * 60 * 60;

function validateInitialize(input) {
  if (input.tokenProgram !== REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';
  if (input.usdcDecimals !== 6) return 'InvalidMintDecimals';
  if (input.treasuryMint !== input.usdcMint) return 'InvalidTreasuryMint';
  if (input.treasuryOwner !== input.treasuryAuthority) return 'InvalidTreasuryOwner';
  if (
    input.maxTotal <= 0 ||
    input.maxPerUser <= 0 ||
    input.maxPerUser > input.maxTotal
  ) return 'InvalidCap';
  if (input.claimWindowSecs < 1 || input.claimWindowSecs > MAX_CLAIM_WINDOW_SECS) return 'InvalidClaimWindow';
  return null;
}

(function run() {
  const base = {
    tokenProgram: REQUIRED_TOKEN_PROGRAM,
    usdcDecimals: 6,
    usdcMint: 'MintA',
    treasuryMint: 'MintA',
    treasuryOwner: 'TreasuryOwnerA',
    treasuryAuthority: 'TreasuryOwnerA',
    maxTotal: 1_000_000,
    maxPerUser: 100_000,
    claimWindowSecs: 3600,
  };

  assert.equal(validateInitialize(base), null);
  assert.equal(validateInitialize({ ...base, tokenProgram: 'TokenzFake' }), 'InvalidTokenProgram');
  assert.equal(validateInitialize({ ...base, usdcDecimals: 9 }), 'InvalidMintDecimals');
  assert.equal(validateInitialize({ ...base, treasuryMint: 'MintB' }), 'InvalidTreasuryMint');
  assert.equal(validateInitialize({ ...base, treasuryOwner: 'OtherOwner' }), 'InvalidTreasuryOwner');
  assert.equal(validateInitialize({ ...base, maxPerUser: 2_000_000 }), 'InvalidCap');
  assert.equal(validateInitialize({ ...base, maxTotal: 0 }), 'InvalidCap');
  assert.equal(validateInitialize({ ...base, claimWindowSecs: 0 }), 'InvalidClaimWindow');
  assert.equal(validateInitialize({ ...base, claimWindowSecs: MAX_CLAIM_WINDOW_SECS + 1 }), 'InvalidClaimWindow');

  console.log('initialize spec tests ok');
})();
