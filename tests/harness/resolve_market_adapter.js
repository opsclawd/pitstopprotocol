const { executeResolveMarket } = require('../../packages/core/src/resolve_market_instruction.cjs');

async function invokeResolveMarketOnProgram(input) {
  return executeResolveMarket(input);
}

module.exports = { invokeResolveMarketOnProgram };
