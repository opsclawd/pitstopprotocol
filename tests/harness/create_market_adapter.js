const { executeCreateMarket } = require('../../packages/core/src/create_market_instruction.cjs');

async function invokeCreateMarketOnProgram(input) {
  return executeCreateMarket(input);
}

module.exports = { invokeCreateMarketOnProgram };
