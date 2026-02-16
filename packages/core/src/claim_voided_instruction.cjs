function validateClaimVoidedInput(input) {
  // CLV-REJ-001 + CLV-ORD-001: status gate must be evaluated before any vault/account usage.
  if (input.marketStatus !== 'Voided') return 'MarketNotVoided';

  // CLV-REJ-002
  if (input.positionClaimed) return 'AlreadyClaimed';

  // CLV-REJ-003 (inclusive end)
  const end = input.resolutionTimestamp + input.claimWindowSecs;
  if (input.nowTs > end) return 'ClaimWindowExpired';

  return null;
}

function executeClaimVoided(input) {
  const err = validateClaimVoidedInput(input);
  if (err) return { ok: false, error: err };

  // Effects: refund full principal.
  const payout = input.positionAmount;

  // Minimal numeric guards for JS reference implementation.
  if (!Number.isInteger(payout) || payout < 0) return { ok: false, error: 'Underflow' };
  if (!Number.isInteger(input.vaultAmount) || input.vaultAmount < payout) return { ok: false, error: 'Underflow' };
  if (!Number.isInteger(input.userUsdcAmount) || input.userUsdcAmount < 0) return { ok: false, error: 'Underflow' };

  const vaultAmount = input.vaultAmount - payout;
  const userUsdcAmount = input.userUsdcAmount + payout;

  const position = {
    ...input.positionState,
    claimed: true,
    payout,
  };

  const event = {
    name: 'Claimed',
    market: input.market,
    user: input.user,
    outcome_id: input.outcomeId,
    payout,
    claimed_at: input.nowTs,
  };

  return { ok: true, position, vaultAmount, userUsdcAmount, event };
}

module.exports = { validateClaimVoidedInput, executeClaimVoided };
