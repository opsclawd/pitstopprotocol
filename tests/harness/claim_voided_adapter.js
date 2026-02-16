const { executeClaimVoided } = require('../../packages/core/src/claim_voided_instruction.cjs');

async function invokeClaimVoidedOnProgram(input) {
  return executeClaimVoided(input);
}

module.exports = { invokeClaimVoidedOnProgram };
