const assert = require('assert');
const constants = require('../../specs/constants.json');
const { invokeSweepRemainingOnProgram } = require('../harness/sweep_remaining_adapter');

(async function run() {
  const resolutionTimestamp = 1_800_000_000;
  const claimWindowSecs = 5000;
  const nowTs = resolutionTimestamp + claimWindowSecs + 1;

  const base = {
    authority: 'AuthA',
    configAuthority: 'AuthA',
    market: 'MarketA',
    marketStatus: 'Resolved',
    nowTs,
    resolutionTimestamp,
    claimWindowSecs,
    vaultAmount: 123,
    treasuryAmount: 1000,
    treasury: 'TreasuryA',
    treasuryMint: 'MintA',
    usdcMint: 'MintA',
    treasuryOwner: 'TreasuryAuthA',
    treasuryAuthority: 'TreasuryAuthA',
    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
    marketState: { status: 'Resolved' },
  };

  // SWP-HP-001
  const ok = await invokeSweepRemainingOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.market.status, 'Swept');
  assert.equal(ok.treasuryAmount, base.treasuryAmount + base.vaultAmount);
  assert.equal(ok.sweptAmount, base.vaultAmount);

  // SWP-SEED-001: modeled vault close uses market PDA signer seeds.
  assert.equal(ok.vaultClosed, true);
  assert.equal(ok.vaultAccountExists, false);
  assert.equal(ok.closeUsedMarketPdaSeeds, true);

  // Event contract
  assert.equal(ok.event.name, 'MarketSweptEvent');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.amount, base.vaultAmount);
  assert.equal(ok.event.to_treasury, base.treasury);
  assert.equal(ok.event.timestamp, nowTs);

  // SWP-REJ-001..004
  const rejCases = [
    [{ authority: 'Other' }, 'Unauthorized'],
    [{ marketStatus: 'Open' }, 'MarketNotResolved'],
    [{ nowTs: resolutionTimestamp + claimWindowSecs }, 'ClaimWindowNotExpired'],
    [{ treasuryMint: 'OtherMint' }, 'InvalidTreasuryMint'],
  ];
  for (const [patch, expected] of rejCases) {
    const out = await invokeSweepRemainingOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined);
  }

  // SWP-IDEM-001: second sweep must fail deterministically via status gate.
  const again = await invokeSweepRemainingOnProgram({ ...base, marketStatus: 'Swept', marketState: { status: 'Swept' } });
  assert.equal(again.ok, false);
  assert.equal(again.error, 'MarketNotResolved');

  // SWP-ADV-001: token program mismatch
  const adv = await invokeSweepRemainingOnProgram({ ...base, tokenProgram: 'TokenzFake' });
  assert.equal(adv.ok, false);
  assert.equal(adv.error, 'InvalidTokenProgram');

  console.log('sweep_remaining conformance tests ok');
})();
