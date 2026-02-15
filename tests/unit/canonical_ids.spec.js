const assert = require('assert');
const crypto = require('crypto');
const vectors = require('../../specs/vectors/canonical_vectors.json');

function canonicalJson(obj) {
  const keys = Object.keys(obj).sort();
  return `{${keys.map((k) => `"${k}":"${obj[k]}"`).join(',')}}`;
}

function sha256Hex(buf) {
  return crypto.createHash('sha256').update(buf).digest('hex');
}

function computeEventIdHex(descriptor) {
  return sha256Hex(Buffer.from(canonicalJson(descriptor), 'utf8'));
}

function computeMarketIdHex(eventIdHex, marketTypeByte, rulesVersion) {
  const event = Buffer.from(eventIdHex, 'hex');
  const b = Buffer.alloc(35);
  event.copy(b, 0);
  b.writeUInt8(marketTypeByte, 32);
  b.writeUInt16LE(rulesVersion, 33);
  return sha256Hex(b);
}

(function run() {
  // Vector A
  assert.equal(canonicalJson(vectors.vectorA.descriptor), vectors.vectorA.canonicalJson);
  assert.equal(computeEventIdHex(vectors.vectorA.descriptor), vectors.vectorA.eventIdHex);

  // Vector B
  assert.equal(
    computeMarketIdHex(vectors.vectorB.eventIdHex, vectors.vectorB.marketTypeByte, vectors.vectorB.rulesVersion),
    vectors.vectorB.marketIdHex
  );

  // Additional deterministic vectors
  const alt = { sport: 'f1', season: '2027', round: '01', session: 'race' };
  const altId = computeEventIdHex(alt);
  assert.equal(altId.length, 64);
  assert.notEqual(altId, vectors.vectorA.eventIdHex);

  const m1 = computeMarketIdHex(altId, 0, 1);
  const m2 = computeMarketIdHex(altId, 0, 1);
  assert.equal(m1, m2, 'market id must be deterministic');

  console.log('canonical_ids unit tests ok');
})();
