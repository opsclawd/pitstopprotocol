const assert = require('assert');

function validateTimestampSeconds(ts) {
  if (!Number.isInteger(ts)) throw new Error('TimestampNotInteger');
  if (ts > 10_000_000_000) throw new Error('TimestampLooksLikeMilliseconds');
  if (ts < 1_577_836_800 || ts > 4_102_444_800) throw new Error('TimestampOutOfBounds');
  return true;
}

(function run() {
  // happy
  assert.equal(validateTimestampSeconds(1_800_000_000), true);

  // rejects
  assert.throws(() => validateTimestampSeconds(1_800_000_000.5), /TimestampNotInteger/);
  assert.throws(() => validateTimestampSeconds(1_800_000_000_000), /TimestampLooksLikeMilliseconds/);
  assert.throws(() => validateTimestampSeconds(1_500_000_000), /TimestampOutOfBounds/);

  // boundaries
  assert.equal(validateTimestampSeconds(1_577_836_800), true);
  assert.equal(validateTimestampSeconds(4_102_444_800), true);

  console.log('timestamp_rules unit tests ok');
})();
