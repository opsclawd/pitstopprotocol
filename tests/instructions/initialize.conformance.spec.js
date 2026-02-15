const assert = require('assert');
const constants = require('../../specs/constants.json');
const { invokeInitializeOnProgram } = require('../harness/initialize_adapter');

(async function run() {
  const nowTs = 1_800_000_000;
  const base = {
    authority: 'AuthA',
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    usdcDecimals: 6,
    usdcMint: 'MintA',
    treasury: 'TreasuryA',
    treasuryMint: 'MintA',
    treasuryOwner: 'TreasuryOwnerA',
    treasuryAuthority: 'TreasuryOwnerA',
    maxTotal: 1_000_000,
    maxPerUser: 100_000,
    claimWindowSecs: 3600,
    nowTs,
  };

  // INIT-HP-001
  const ok = await invokeInitializeOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.config.authority, base.authority);
  assert.equal(ok.config.oracle, base.authority);
  assert.equal(ok.config.usdcMint, base.usdcMint);
  assert.equal(ok.config.treasury, base.treasury);
  assert.equal(ok.config.treasuryAuthority, base.treasuryAuthority);
  assert.equal(ok.config.feeBps, 0);
  assert.equal(ok.config.paused, false);
  assert.equal(ok.config.maxTotalPoolPerMarket, base.maxTotal);
  assert.equal(ok.config.maxBetPerUserPerMarket, base.maxPerUser);
  assert.equal(ok.config.claimWindowSecs, base.claimWindowSecs);
  assert.equal(ok.config.tokenProgram, constants.REQUIRED_TOKEN_PROGRAM);

  assert.equal(ok.event.name, 'ConfigInitialized');
  assert.equal(ok.event.authority, base.authority);
  assert.equal(ok.event.oracle, base.authority);
  assert.equal(ok.event.usdc_mint, base.usdcMint);
  assert.equal(ok.event.treasury, base.treasury);
  assert.equal(ok.event.fee_bps, 0);
  assert.equal(ok.event.timestamp, nowTs);

  // INIT-REJ-001..006
  const cases = [
    [{ tokenProgram: 'TokenzFake' }, 'InvalidTokenProgram'],
    [{ usdcDecimals: 9 }, 'InvalidMintDecimals'],
    [{ treasuryMint: 'MintB' }, 'InvalidTreasuryMint'],
    [{ treasuryOwner: 'OtherOwner' }, 'InvalidTreasuryOwner'],
    [{ maxPerUser: 2_000_000 }, 'InvalidCap'],
    [{ claimWindowSecs: constants.MAX_CLAIM_WINDOW_SECS + 1 }, 'InvalidClaimWindow'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokeInitializeOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
  }

  console.log('initialize conformance tests ok');
})();
