const { computeMarketIdHex } = require('./protocol_primitives.cjs');
const constants = require('../../../specs/constants.json');

const MAX_OUTCOMES = 100;
const SUPPORTED_MARKET_TYPE = 0;
const SUPPORTED_RULES_VERSION = 1;

function validateCreateMarketInput(input) {
  if (input.authority !== input.configAuthority) return 'Unauthorized';
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';
  if (input.lockTimestamp <= input.nowTs) return 'LockInPast';
  if (input.maxOutcomes === 0) return 'ZeroOutcomes';
  if (input.maxOutcomes > MAX_OUTCOMES) return 'TooManyOutcomes';
  if (input.marketType !== SUPPORTED_MARKET_TYPE) return 'UnsupportedMarketType';
  if (input.rulesVersion !== SUPPORTED_RULES_VERSION) return 'UnsupportedRulesVersion';

  const recomputed = computeMarketIdHex(input.eventIdHex, input.marketType, input.rulesVersion);
  if (recomputed !== input.marketIdHex) return 'InvalidMarketId';

  return null;
}

function executeCreateMarket(input) {
  const err = validateCreateMarketInput(input);
  if (err) return { ok: false, error: err };

  const market = {
    marketIdHex: input.marketIdHex,
    eventIdHex: input.eventIdHex,
    lockTimestamp: input.lockTimestamp,
    outcomeCount: 0,
    maxOutcomes: input.maxOutcomes,
    totalPool: 0,
    status: 'Seeding',
    resolvedOutcome: null,
    resolutionPayloadHash: '0'.repeat(64),
    resolutionTimestamp: 0,
    marketType: input.marketType,
    rulesVersion: input.rulesVersion,
    vault: input.vault,
  };

  const event = {
    name: 'MarketCreated',
    market: input.market,
    market_id: input.marketIdHex,
    event_id: input.eventIdHex,
    lock_timestamp: input.lockTimestamp,
    max_outcomes: input.maxOutcomes,
    market_type: input.marketType,
    rules_version: input.rulesVersion,
    timestamp: input.nowTs,
  };

  return { ok: true, market, event };
}

module.exports = {
  MAX_OUTCOMES,
  SUPPORTED_MARKET_TYPE,
  SUPPORTED_RULES_VERSION,
  validateCreateMarketInput,
  executeCreateMarket,
};
