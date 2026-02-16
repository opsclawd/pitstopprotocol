const assert = require('assert');
const { invokeClaimVoidedOnProgram } = require('../harness/claim_voided_adapter');

(async function run() {
  const nowTs = 1_800_000_100;
  const base = {
    market: 'MarketA',
    user: 'UserA',
    outcomeId: 7,

    marketState: { status: 'Voided' },

    resolutionTimestamp: 1_800_000_000,
    claimWindowSecs: 3600,
    nowTs,

    vaultAmount: 10_000,
    userUsdcAmount: 1_000,

    positionState: { claimed: false, payout: 0, amount: 250 },
  };

  // CLV-HP-001 + CLV-INV-001
  const ok = await invokeClaimVoidedOnProgram(base);
  assert.equal(ok.ok, true);
  assert.equal(ok.position.claimed, true);
  assert.equal(ok.position.payout, base.positionState.amount);
  assert.equal(ok.vaultAmount, base.vaultAmount - base.positionState.amount);
  assert.equal(ok.userUsdcAmount, base.userUsdcAmount + base.positionState.amount);

  assert.equal(ok.event.name, 'Claimed');
  assert.equal(ok.event.market, base.market);
  assert.equal(ok.event.user, base.user);
  assert.equal(ok.event.outcome_id, base.outcomeId);
  assert.equal(ok.event.payout, base.positionState.amount);
  assert.equal(ok.event.claimed_at, nowTs);

  // CLV-REJ-001..003
  const cases = [
    [{ marketState: { status: 'Resolved' } }, 'MarketNotVoided'],
    [{ positionState: { ...base.positionState, claimed: true } }, 'AlreadyClaimed'],
    [{ nowTs: base.resolutionTimestamp + base.claimWindowSecs + 1 }, 'ClaimWindowExpired'],
  ];
  for (const [patch, expected] of cases) {
    const out = await invokeClaimVoidedOnProgram({ ...base, ...patch });
    assert.equal(out.ok, false);
    assert.equal(out.error, expected);
    assert.equal(out.event, undefined);
  }

  // CLV-ORD-001: post-sweep claim must fail by status error before any vault usage.
  // Here vaultAmount is intentionally missing; correct behavior is a deterministic MarketNotVoided.
  const swept = await invokeClaimVoidedOnProgram({ ...base, marketState: { status: 'Swept' }, vaultAmount: undefined });
  assert.equal(swept.ok, false);
  assert.equal(swept.error, 'MarketNotVoided');

  console.log('claim_voided conformance tests ok');
})();
