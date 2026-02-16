const { executeLockMarket } = require('../../packages/core/src/lock_market_instruction.cjs');

async function invokeLockMarketOnProgram(input) {
  return executeLockMarket(input);
}

module.exports = { invokeLockMarketOnProgram };
