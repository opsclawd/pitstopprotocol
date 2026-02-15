const assert = require('assert');
const constants = require('../../specs/constants.json');
const { HarnessConfig } = require('./config');
const { unixNowSeconds, assertSecondsTimestamp, buildLockWindow } = require('./time');
const { getHarnessProvider } = require('./provider');
const { expectedUsdcFixtureShape } = require('../fixtures/usdc_fixture');

(function run() {
  assert.equal(HarnessConfig.usdcDecimals, 6);
  assert.equal(HarnessConfig.tokenProgram, constants.REQUIRED_TOKEN_PROGRAM);

  const now = unixNowSeconds();
  assertSecondsTimestamp(now, 'now');

  const { lock } = buildLockWindow(120);
  assert.ok(lock > now, 'lock must be in the future');

  const provider = getHarnessProvider();
  assert.ok(provider.rpcUrl.startsWith('http'), 'rpc url should be http(s)');

  const usdc = expectedUsdcFixtureShape();
  assert.equal(usdc.decimals, 6, 'USDC fixture must enforce 6 decimals');

  console.log('harness smoke ok');
})();
