const { executeCancelMarket } = require('../../packages/core/src/cancel_market_instruction.cjs');

async function invokeCancelMarketOnProgram(input) {
  return executeCancelMarket(input);
}

module.exports = { invokeCancelMarketOnProgram };
