const assert = require('assert');
const crypto = require('crypto');
const vectors = require('../specs/vectors/canonical_vectors.json');

function canonicalJson(obj){
  const keys = Object.keys(obj).sort();
  return `{${keys.map(k=>`"${k}":"${obj[k]}"`).join(',')}}`;
}
function sha256Hex(buf){ return crypto.createHash('sha256').update(buf).digest('hex'); }
function marketId(eventIdHex, marketType, rulesVersion){
  const event = Buffer.from(eventIdHex, 'hex');
  const b = Buffer.alloc(35);
  event.copy(b,0);
  b.writeUInt8(marketType,32);
  b.writeUInt16LE(rulesVersion,33);
  return sha256Hex(b);
}

const cj = canonicalJson(vectors.vectorA.descriptor);
assert.equal(cj, vectors.vectorA.canonicalJson);
assert.equal(sha256Hex(Buffer.from(cj,'utf8')), vectors.vectorA.eventIdHex);
assert.equal(
  marketId(vectors.vectorB.eventIdHex, vectors.vectorB.marketTypeByte, vectors.vectorB.rulesVersion),
  vectors.vectorB.marketIdHex
);
console.log('canonical vectors ok');
