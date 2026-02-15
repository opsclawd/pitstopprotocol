/**
 * USDC fixture contract (locked interface for #58/#59).
 */
const constants = require('../../specs/constants.json');

function usdcFixtureSpec() {
  return {
    decimals: constants.USDC_DECIMALS,
    mintAddress: null, // null until adapter creates real mint
    source: 'adapter',
  };
}

async function getOrCreateUsdcMint(adapter) {
  if (!adapter || typeof adapter.getOrCreateUsdcMint !== 'function') {
    return usdcFixtureSpec();
  }
  const out = await adapter.getOrCreateUsdcMint({ decimals: constants.USDC_DECIMALS });
  return {
    decimals: constants.USDC_DECIMALS,
    mintAddress: out.mintAddress,
    source: 'adapter',
  };
}

module.exports = { usdcFixtureSpec, getOrCreateUsdcMint };
