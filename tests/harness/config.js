const constants = require('../../specs/constants.json');

const HarnessConfig = {
  fixedNowSecs: process.env.HARNESS_NOW_SECS ? Number(process.env.HARNESS_NOW_SECS) : null,
  rpcUrl: process.env.SOLANA_RPC_URL || 'http://127.0.0.1:8899',
  wsUrl: process.env.SOLANA_WS_URL || 'ws://127.0.0.1:8900',
  commitment: process.env.SOLANA_COMMITMENT || 'confirmed',
  usdcDecimals: constants.USDC_DECIMALS,
  maxClaimWindowSecs: constants.MAX_CLAIM_WINDOW_SECS,
  tokenProgram: constants.REQUIRED_TOKEN_PROGRAM,
  deterministicSeed: process.env.HARNESS_SEED || "pitstop-harness-seed-v1",
};

module.exports = { HarnessConfig };
