const assert = require('assert');
const crypto = require('crypto');
function marketId(eventIdHex, marketType, rulesVersion){
  const event = Buffer.from(eventIdHex, 'hex');
  const b = Buffer.alloc(35);
  event.copy(b,0);
  b.writeUInt8(marketType,32);
  b.writeUInt16LE(rulesVersion,33);
  return crypto.createHash('sha256').update(b).digest('hex');
}
assert.equal(marketId('00'.repeat(32),0,1).length,64);
console.log('canonical smoke ok');
