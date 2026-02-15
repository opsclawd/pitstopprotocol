const assert = require('assert');

function assertEq(actual, expected, msg) {
  assert.deepEqual(actual, expected, msg);
}

function assertGt(a, b, msg) {
  assert.ok(a > b, msg || `${a} must be > ${b}`);
}

function assertGte(a, b, msg) {
  assert.ok(a >= b, msg || `${a} must be >= ${b}`);
}

module.exports = { assertEq, assertGt, assertGte };
