const assert = require('assert');
const { validateTimestampSeconds } = require('../../packages/core/src/protocol_primitives.cjs');

(function run() {
  assert.equal(validateTimestampSeconds(1_800_000_000), true);

  assert.throws(() => validateTimestampSeconds(1_800_000_000.5), /TimestampNotInteger/);
  assert.throws(() => validateTimestampSeconds(1_800_000_000_000), /TimestampLooksLikeMilliseconds/);
  assert.throws(() => validateTimestampSeconds(1_500_000_000), /TimestampOutOfBounds/);

  assert.equal(validateTimestampSeconds(1_577_836_800), true);
  assert.equal(validateTimestampSeconds(4_102_444_800), true);
  assert.throws(() => validateTimestampSeconds(4_102_444_801), /TimestampOutOfBounds/);

  console.log('timestamp_rules unit tests ok');
})();
