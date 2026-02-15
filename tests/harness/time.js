const { HarnessConfig } = require('./config');

function unixNowSeconds(opts = {}) {
  const injected = opts.nowSeconds ?? HarnessConfig.fixedNowSecs;
  if (injected !== null && injected !== undefined) return Number(injected);
  return Math.floor(Date.now() / 1000);
}

function assertSecondsTimestamp(ts, label = 'timestamp') {
  if (!Number.isInteger(ts)) throw new Error(`${label} must be integer seconds`);
  if (ts > 10_000_000_000) throw new Error(`${label} appears to be milliseconds`);
  if (ts < 1_577_836_800 || ts > 4_102_444_800) throw new Error(`${label} out of sane bounds`);
}

function buildLockWindow(offsetSecs = 300, opts = {}) {
  const now = unixNowSeconds(opts);
  const lock = now + offsetSecs;
  return { now, lock };
}

module.exports = { unixNowSeconds, assertSecondsTimestamp, buildLockWindow };
