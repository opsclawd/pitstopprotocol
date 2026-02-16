const { executeClaimResolved } = require('../../packages/core/src/claim_resolved_instruction.cjs');

async function invokeClaimResolvedOnProgram(input) {
  return executeClaimResolved(input);
}

module.exports = { invokeClaimResolvedOnProgram };
