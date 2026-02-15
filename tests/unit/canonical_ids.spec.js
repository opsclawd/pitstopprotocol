const assert = require('assert');
const vectors = require('../../specs/vectors/canonical_vectors.json');
const {
  canonicalJson,
  computeEventIdHex,
  computeMarketIdHex,
} = require('../../packages/core/src/protocol_primitives.cjs');

(function run() {
  assert.equal(canonicalJson(vectors.vectorA.descriptor), vectors.vectorA.canonicalJson);
  assert.equal(computeEventIdHex(vectors.vectorA.descriptor), vectors.vectorA.eventIdHex);

  assert.equal(
    computeMarketIdHex(vectors.vectorB.eventIdHex, vectors.vectorB.marketTypeByte, vectors.vectorB.rulesVersion),
    vectors.vectorB.marketIdHex
  );

  const alt = { sport: 'f1', season: '2027', round: '01', session: 'race' };
  const altId = computeEventIdHex(alt);
  assert.equal(altId.length, 64);
  assert.notEqual(altId, vectors.vectorA.eventIdHex);

  const m1 = computeMarketIdHex(altId, 0, 1);
  const m2 = computeMarketIdHex(altId, 0, 1);
  assert.equal(m1, m2, 'market id must be deterministic');

  assert.throws(() => computeMarketIdHex('abc', 0, 1), /InvalidEventIdHex/);
  assert.throws(() => computeMarketIdHex(vectors.vectorB.eventIdHex, 256, 1), /InvalidMarketTypeByte/);
  assert.throws(() => computeMarketIdHex(vectors.vectorB.eventIdHex, 0, 65536), /InvalidRulesVersion/);

  console.log('canonical_ids unit tests ok');
})();
