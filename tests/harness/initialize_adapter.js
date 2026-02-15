class NotImplementedConformanceAdapter extends Error {}

async function invokeInitializeOnProgram(_input) {
  throw new NotImplementedConformanceAdapter('Initialize on-chain conformance adapter not wired yet');
}

module.exports = { invokeInitializeOnProgram, NotImplementedConformanceAdapter };
