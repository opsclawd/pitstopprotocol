/**
 * Harness provider abstraction (locked interface for #58/#59).
 *
 * Returned object contract:
 * - rpcUrl: string
 * - wsUrl: string
 * - commitment: string
 * - deterministicSeed: string
 * - getConnection(): { rpcUrl, wsUrl, commitment } (adapter placeholder)
 */
const { HarnessConfig } = require('./config');

function getHarnessProvider() {
  return {
    rpcUrl: HarnessConfig.rpcUrl,
    wsUrl: HarnessConfig.wsUrl,
    commitment: HarnessConfig.commitment,
    deterministicSeed: HarnessConfig.deterministicSeed,
    getConnection() {
      return {
        rpcUrl: HarnessConfig.rpcUrl,
        wsUrl: HarnessConfig.wsUrl,
        commitment: HarnessConfig.commitment,
      };
    },
  };
}

module.exports = { getHarnessProvider };
