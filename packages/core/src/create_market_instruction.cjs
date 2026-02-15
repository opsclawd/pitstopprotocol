const { computeMarketIdHex } = require('./protocol_primitives.cjs');
const constants = require('../../../specs/constants.json');

const MAX_OUTCOMES = 100;
const SUPPORTED_MARKET_TYPE = 0;
const SUPPORTED_RULES_VERSION = 1;

function validateCreateMarketInput(input) {
  // CRM-REJ-001: only config authority can create markets.
  if (input.authority !== input.configAuthority) return 'Unauthorized';
  // CRM-REJ-002: token program is pinned to configured SPL Token v1.
  if (input.tokenProgram !== constants.REQUIRED_TOKEN_PROGRAM) return 'InvalidTokenProgram';
  // CRM-REJ-003: market lock time must be strictly in the future.
  if (input.lockTimestamp <= input.nowTs) return 'LockInPast';
  // CRM-REJ-004a: zero outcomes is invalid.
  if (input.maxOutcomes === 0) return 'ZeroOutcomes';
  // CRM-REJ-004b: cap outcomes to deterministic MAX_OUTCOMES bound.
  if (input.maxOutcomes > MAX_OUTCOMES) return 'TooManyOutcomes';
  // CRM-REJ-005a: MVP currently supports market_type=Winner(0) only.
  if (input.marketType !== SUPPORTED_MARKET_TYPE) return 'UnsupportedMarketType';
  // CRM-REJ-005b: MVP currently supports rules_version=1 only.
  if (input.rulesVersion !== SUPPORTED_RULES_VERSION) return 'UnsupportedRulesVersion';

  // CRM-REJ-006: recompute on-chain-equivalent market_id to prevent canonicalization drift.
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
    // create_market always initializes market lifecycle at Seeding.
    status: 'Seeding',
    resolvedOutcome: null,
    resolutionPayloadHash: '0'.repeat(64),
    resolutionTimestamp: 0,
    marketType: input.marketType,
    rulesVersion: input.rulesVersion,
    vault: input.vault,
  };

  // Must emit MarketCreated only after successful state/vault initialization.
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
