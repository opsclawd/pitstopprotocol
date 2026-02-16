const { executeVoidMarket } = require('../../packages/core/src/void_market_instruction.cjs');

async function invokeVoidMarketOnProgram(input) {
  return executeVoidMarket(input);
}

module.exports = { invokeVoidMarketOnProgram };
