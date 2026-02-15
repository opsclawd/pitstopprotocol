const { executePlaceBet } = require('../../packages/core/src/place_bet_instruction.cjs');

async function invokePlaceBetOnProgram(input) {
  return executePlaceBet(input);
}

module.exports = { invokePlaceBetOnProgram };
