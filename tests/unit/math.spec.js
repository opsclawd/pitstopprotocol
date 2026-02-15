const assert = require('assert');
const {
  computeFee,
  computePrizePool,
  computePayout,
} = require('../../packages/core/src/protocol_primitives.cjs');

(function run() {
  assert.equal(computeFee(1_000_000, 0), 0);
  assert.equal(computeFee(1_000_000, 500), 50_000);
  assert.equal(computeFee(1_000_001, 500), 50_000);
  assert.throws(() => computeFee(100, 10_001), /FeeBpsOutOfRange/);
  assert.throws(() => computeFee(-1, 100), /InvalidTotalPool/);

  assert.equal(computePrizePool(1_000_000, 500), 950_000);

  assert.equal(computePayout(100, 950, 1000), 95);
  assert.equal(computePayout(1, 3, 2), 1);
  assert.throws(() => computePayout(1, 100, 0), /DivisionByZero/);

  const prize = 100;
  const winnerPool = 3;
  const payouts = [1, 1, 1].map((amt) => computePayout(amt, prize, winnerPool));
  const sumPayouts = payouts.reduce((a, b) => a + b, 0);
  const dust = prize - sumPayouts;
  assert.equal(sumPayouts, 99);
  assert.equal(dust, 1);
  assert.ok(dust <= payouts.length - 1);

  console.log('math unit tests ok');
})();
