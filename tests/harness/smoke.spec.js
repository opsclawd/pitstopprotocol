const assert = require('assert');
const constants = require('../../specs/constants.json');
const { HarnessConfig } = require('./config');
const { unixNowSeconds, assertSecondsTimestamp, buildLockWindow } = require('./time');
const { getHarnessProvider } = require('./provider');
const { usdcFixtureSpec } = require('../fixtures/usdc_fixture');

(function run() {
  assert.equal(HarnessConfig.usdcDecimals, 6);
  assert.equal(HarnessConfig.tokenProgram, constants.REQUIRED_TOKEN_PROGRAM);

  const frozenNow = 1_800_000_000;
  const now = unixNowSeconds({ nowSeconds: frozenNow });
  assert.equal(now, frozenNow, 'time helper must support deterministic injected now');
  assertSecondsTimestamp(now, 'now');

  const { lock } = buildLockWindow(120, { nowSeconds: frozenNow });
  assert.equal(lock, frozenNow + 120, 'lock window must be deterministic under injected now');

  const provider = getHarnessProvider();
  assert.ok(provider.rpcUrl.startsWith('http'), 'rpc url should be http(s)');
  assert.equal(typeof provider.getConnection, 'function', 'provider must expose getConnection()');
  assert.equal(typeof provider.deterministicSeed, 'string', 'provider must expose deterministic seed');

  const usdc = usdcFixtureSpec();
  assert.equal(usdc.decimals, 6, 'USDC fixture must enforce 6 decimals');
  assert.equal(usdc.mintAddress, null, 'fixture returns null mint before adapter wiring');

  console.log('harness smoke ok');
})();
