const { executeAddOutcome } = require('../../packages/core/src/add_outcome_instruction.cjs');

async function invokeAddOutcomeOnProgram(input) {
  return executeAddOutcome(input);
}

module.exports = { invokeAddOutcomeOnProgram };
