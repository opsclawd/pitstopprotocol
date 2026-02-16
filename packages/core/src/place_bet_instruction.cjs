const constants = require('../../../specs/constants.json');

function validatePlaceBetInput(input) {
  if (input.configPaused) return 'ProtocolPaused'; // PBT-REJ-001
  if (input.marketState.status !== 'Open') return 'MarketNotOpen'; // PBT-REJ-002
  if (input.nowTs >= input.marketState.lockTimestamp) return 'BettingClosed'; // PBT-REJ-003
  if (!Number.isInteger(input.outcomeId) || input.outcomeId < 0 || input.outcomeId > 99) return 'InvalidOutcomeId'; // PBT-REJ-004
  if (input.marketState.outcomeCount !== input.marketState.maxOutcomes) return 'MarketNotReady'; // PBT-REJ-005
  if (!Number.isInteger(input.amount) || input.amount <= 0) return 'ZeroAmount'; // PBT-REJ-006
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram'; // PBT-REJ-010

  if (
    !input.outcomePoolState ||
    input.outcomePoolState.market !== input.market ||
    input.outcomePoolState.outcomeId !== input.outcomeId
  ) {
    return 'OutcomeMismatch'; // PBT-REJ-009 (wrapped deterministic mismatch)
  }

  const nextMarketTotal = input.marketState.totalPool + input.amount;
  if (nextMarketTotal > input.maxTotalPoolPerMarket) return 'MarketCapExceeded'; // PBT-REJ-007

  const nextUserPosition = input.positionState.amount + input.amount;
  if (nextUserPosition > input.maxBetPerUserPerMarket) return 'UserBetCapExceeded'; // PBT-REJ-008

  return null;
}

function executePlaceBet(input) {
  const err = validatePlaceBetInput(input);
  if (err) return { ok: false, error: err };

  const nextMarketPoolTotal = input.marketState.totalPool + input.amount;
  const outcomePoolAmount = input.outcomePoolState.poolAmount + input.amount;
  const nextPositionStake = input.positionState.amount + input.amount;

  const market = { ...input.marketState, totalPool: nextMarketPoolTotal };
  const outcomePool = { ...input.outcomePoolState, poolAmount: outcomePoolAmount };
  const position = { ...input.positionState, amount: nextPositionStake };
  const vaultAmount = input.vaultAmount + input.amount;

  const event = {
    name: 'BetPlaced',
    market: input.market,
    user: input.user,
    outcome_id: input.outcomeId,
    amount: input.amount,
    market_total_pool: nextMarketPoolTotal,
    outcome_pool_amount: outcomePoolAmount,
    timestamp: input.nowTs,
  };

  return { ok: true, market, outcomePool, position, vaultAmount, event };
}

module.exports = { validatePlaceBetInput, executePlaceBet };
