const { executeSweepRemaining } = require('../../packages/core/src/sweep_remaining_instruction.cjs');

async function invokeSweepRemainingOnProgram(input) {
  return executeSweepRemaining(input);
}

module.exports = { invokeSweepRemainingOnProgram };
