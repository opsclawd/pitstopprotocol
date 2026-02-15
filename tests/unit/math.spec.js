const assert = require('assert');

function computeFee(totalPool, feeBps) {
  if (feeBps < 0 || feeBps > 10_000) throw new Error('FeeBpsOutOfRange');
  return Math.floor((totalPool * feeBps) / 10_000);
}

function computePrizePool(totalPool, feeBps) {
  const fee = computeFee(totalPool, feeBps);
  return totalPool - fee;
}

function computePayout(positionAmount, prizePool, winnerPool) {
  if (winnerPool === 0) throw new Error('DivisionByZero');
  return Math.floor((positionAmount * prizePool) / winnerPool);
}

(function run() {
  // fee math
  assert.equal(computeFee(1_000_000, 0), 0);
  assert.equal(computeFee(1_000_000, 500), 50_000);
  assert.equal(computeFee(1_000_001, 500), 50_000, 'fee should floor');
  assert.throws(() => computeFee(100, 10_001), /FeeBpsOutOfRange/);

  // prize pool
  assert.equal(computePrizePool(1_000_000, 500), 950_000);

  // payout math
  assert.equal(computePayout(100, 950, 1000), 95);
  assert.equal(computePayout(1, 3, 2), 1, 'floor division behavior');
  assert.throws(() => computePayout(1, 100, 0), /DivisionByZero/);

  // dust behavior example
  const prize = 100;
  const winnerPool = 3;
  const payouts = [1, 1, 1].map((amt) => computePayout(amt, prize, winnerPool)); // [33,33,33]
  const sumPayouts = payouts.reduce((a, b) => a + b, 0);
  const dust = prize - sumPayouts;
  assert.equal(sumPayouts, 99);
  assert.equal(dust, 1);
  assert.ok(dust <= payouts.length - 1, 'dust bound sanity');

  console.log('math unit tests ok');
})();
