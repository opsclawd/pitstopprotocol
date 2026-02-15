/**
 * USDC fixture contract (6 decimals).
 *
 * NOTE: Implementation is intentionally adapter-driven. The concrete chain
 * wiring is added when protocol instruction tests are introduced.
 */
const constants = require('../../specs/constants.json');

function expectedUsdcFixtureShape() {
  return {
    decimals: constants.USDC_DECIMALS,
    mintAddress: 'TO_BE_CREATED_IN_INTEGRATION_TESTS',
  };
}

module.exports = { expectedUsdcFixtureShape };
