/**
 * Harness provider abstraction.
 *
 * This intentionally does not hard-bind to Anchor runtime yet.
 * PR #57 defines the interface and deterministic fixtures contract,
 * PR #58+ will wire this to concrete validator/program clients.
 */
const { HarnessConfig } = require('./config');

function getHarnessProvider() {
  return {
    rpcUrl: HarnessConfig.rpcUrl,
    wsUrl: HarnessConfig.wsUrl,
    commitment: HarnessConfig.commitment,
  };
}

module.exports = { getHarnessProvider };
