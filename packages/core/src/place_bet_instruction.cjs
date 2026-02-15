const constants = require('../../../specs/constants.json');

function validatePlaceBetInput(input) {
  if (input.configPaused) return 'ProtocolPaused'; // PBT-REJ-001
  if (input.marketStatus !== 'Open') return 'MarketNotOpen'; // PBT-REJ-002
  if (input.nowTs >= input.marketLockTimestamp) return 'BettingClosed'; // PBT-REJ-003
  if (!Number.isInteger(input.outcomeId) || input.outcomeId < 0 || input.outcomeId > 99) return 'InvalidOutcomeId'; // PBT-REJ-004
  if (input.marketOutcomeCount !== input.marketMaxOutcomes) return 'MarketNotReady'; // PBT-REJ-005
  if (!Number.isInteger(input.amount) || input.amount <= 0) return 'ZeroAmount'; // PBT-REJ-006
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram'; // PBT-REJ-010

  if (!input.outcomePoolExists || input.outcomePoolMarket !== input.market || input.outcomePoolOutcomeId !== input.outcomeId) {
    return 'OutcomeMismatch'; // PBT-REJ-009 (wrapped deterministic mismatch)
  }

  const nextMarketTotal = input.marketTotalPool + input.amount;
  if (nextMarketTotal > input.maxTotalPoolPerMarket) return 'MarketCapExceeded'; // PBT-REJ-007

  const nextUserPosition = input.userPositionAmount + input.amount;
  if (nextUserPosition > input.maxBetPerUserPerMarket) return 'UserBetCapExceeded'; // PBT-REJ-008

  return null;
}

function executePlaceBet(input) {
  const err = validatePlaceBetInput(input);
  if (err) return { ok: false, error: err };

  const marketTotalPool = input.marketTotalPool + input.amount;
  const outcomePoolAmount = input.outcomePoolAmount + input.amount;
  const positionAmount = input.userPositionAmount + input.amount;

  const market = { ...input.marketState, totalPool: marketTotalPool };
  const outcomePool = { ...input.outcomePoolState, poolAmount: outcomePoolAmount };
  const position = { ...input.positionState, amount: positionAmount };
  const vaultAmount = input.vaultAmount + input.amount;

  const event = {
    name: 'BetPlaced',
    market: input.market,
    user: input.user,
    outcome_id: input.outcomeId,
    amount: input.amount,
    market_total_pool: marketTotalPool,
    outcome_pool_amount: outcomePoolAmount,
    timestamp: input.nowTs,
  };

  return { ok: true, market, outcomePool, position, vaultAmount, event };
}

module.exports = { validatePlaceBetInput, executePlaceBet };
