const { executeFinalizeSeeding } = require('../../packages/core/src/finalize_seeding_instruction.cjs');

async function invokeFinalizeSeedingOnProgram(input) {
  return executeFinalizeSeeding(input);
}

module.exports = { invokeFinalizeSeedingOnProgram };
