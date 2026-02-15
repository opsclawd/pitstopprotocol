const { executeInitialize } = require('../../packages/core/src/initialize_instruction.cjs');

class NotImplementedConformanceAdapter extends Error {}

async function invokeInitializeOnProgram(input) {
  // Conformance bridge for #59: deterministic implementation bridge until on-chain harness wiring lands.
  return executeInitialize(input);
}

module.exports = { invokeInitializeOnProgram, NotImplementedConformanceAdapter };
