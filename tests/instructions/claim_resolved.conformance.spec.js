const assert = require('assert');
const constants = require('../../specs/constants.json');
const { invokeClaimResolvedOnProgram } = require('../harness/claim_resolved_adapter');

(async function run() {
  const nowTs = 1_800_000_100;

  const base = {
    user: 'UserA',
    market: 'MarketA',
    outcomeId: 1,

    marketState: { status: 'Resolved', totalPool: 1_000_000 },
    winningOutcomeId: 1,

    nowTs,
    resolutionTimestamp: 1_800_000_000,
    claimWindowSecs: 3600,

    feeBps: 200, // 2%

    positionExists: true,
    positionState: { amount: 100_000, claimed: false, payout: 0 },

    outcomePoolState: { market: 'MarketA', outcomeId: 1, poolAmount: 500_000 },

    userUsdcAmount: 10,
    vaultAmount: 1_000_000,

    tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
  };

  // CLR-HP-001
  {
    const out = await invokeClaimResolvedOnProgram(base);
    assert.equal(out.ok, true);
    assert.equal(out.position.claimed, true);
    assert.equal(out.position.payout, 196_000);

    assert.equal(out.userUsdcAmount, base.userUsdcAmount + out.position.payout);
    assert.equal(out.vaultAmount, base.vaultAmount - out.position.payout);

    assert.equal(out.event.name, 'Claimed');
    assert.equal(out.event.market, base.market);
    assert.equal(out.event.user, base.user);
    assert.equal(out.event.outcome_id, base.outcomeId);
    assert.equal(out.event.payout, out.position.payout);
    assert.equal(out.event.claimed_at, nowTs);
  }

  // CLR-HP-002
  {
    const loser = {
      ...base,
      outcomeId: 0,
      outcomePoolState: { ...base.outcomePoolState, outcomeId: 0 },
      winningOutcomeId: 1,
    };
    const out = await invokeClaimResolvedOnProgram(loser);
    assert.equal(out.ok, true);
    assert.equal(out.position.claimed, true);
    assert.equal(out.position.payout, 0);
    assert.equal(out.vaultAmount, loser.vaultAmount);
    assert.equal(out.userUsdcAmount, loser.userUsdcAmount);
    assert.equal(out.event.payout, 0);
  }

  // CLR-HP-003
  {
    const feeCase = {
      ...base,
      feeBps: 333,
      marketState: { ...base.marketState, totalPool: 101 },
      positionState: { amount: 1, claimed: false, payout: 0 },
      outcomePoolState: { ...base.outcomePoolState, poolAmount: 3 },
      vaultAmount: 101,
    };
    const out = await invokeClaimResolvedOnProgram(feeCase);
    assert.equal(out.ok, true);
    assert.equal(out.position.payout, 32);
    assert.equal(out.vaultAmount, 69);
  }

  // CLR-REJ-001..004
  {
    const cases = [
      [{ marketState: { ...base.marketState, status: 'Locked' } }, 'MarketNotResolved'],
      [{ positionState: { ...base.positionState, claimed: true } }, 'AlreadyClaimed'],
      [{ nowTs: base.resolutionTimestamp + base.claimWindowSecs + 1 }, 'ClaimWindowExpired'],
      [{ outcomePoolState: null }, 'OutcomeMismatch'],
    ];

    for (const [patch, expected] of cases) {
      const out = await invokeClaimResolvedOnProgram({ ...base, ...patch });
      assert.equal(out.ok, false);
      assert.equal(out.error, expected);
      assert.equal(out.event, undefined);
    }
  }

  // CLR-ORD-001
  {
    const swept = {
      ...base,
      marketState: { ...base.marketState, status: 'Swept' },
      outcomePoolState: null,
      tokenProgram: 'WrongTokenProgram',
    };
    const out = await invokeClaimResolvedOnProgram(swept);
    assert.equal(out.ok, false);
    assert.equal(out.error, 'MarketNotResolved');
  }

  // CLR-INV-001
  {
    const out = await invokeClaimResolvedOnProgram({ ...base, positionState: { ...base.positionState, claimed: true } });
    assert.equal(out.ok, false);
    assert.equal(out.error, 'AlreadyClaimed');
  }

  // CLR-INV-002
  {
    const winnerOut = await invokeClaimResolvedOnProgram(base);
    assert.equal(winnerOut.ok, true);
    assert.equal(winnerOut.vaultAmount, base.vaultAmount - winnerOut.position.payout);

    const loser = {
      ...base,
      outcomeId: 0,
      outcomePoolState: { ...base.outcomePoolState, outcomeId: 0 },
      winningOutcomeId: 1,
    };
    const loserOut = await invokeClaimResolvedOnProgram(loser);
    assert.equal(loserOut.ok, true);
    assert.equal(loserOut.vaultAmount, loser.vaultAmount);
  }

  console.log('claim_resolved conformance tests ok');
})();
