const crypto = require('crypto');

function canonicalJson(descriptor) {
  const keys = Object.keys(descriptor).sort();
  return `{${keys.map((k) => `"${k}":"${descriptor[k]}"`).join(',')}}`;
}

function sha256Hex(buf) {
  return crypto.createHash('sha256').update(buf).digest('hex');
}

function computeEventIdHex(descriptor) {
  return sha256Hex(Buffer.from(canonicalJson(descriptor), 'utf8'));
}

function computeMarketIdHex(eventIdHex, marketTypeByte, rulesVersion) {
  if (!/^[0-9a-fA-F]{64}$/.test(eventIdHex)) throw new Error('InvalidEventIdHex');
  if (!Number.isInteger(marketTypeByte) || marketTypeByte < 0 || marketTypeByte > 255) {
    throw new Error('InvalidMarketTypeByte');
  }
  if (!Number.isInteger(rulesVersion) || rulesVersion < 0 || rulesVersion > 65535) {
    throw new Error('InvalidRulesVersion');
  }
  const event = Buffer.from(eventIdHex, 'hex');
  const b = Buffer.alloc(35);
  event.copy(b, 0);
  b.writeUInt8(marketTypeByte, 32);
  b.writeUInt16LE(rulesVersion, 33);
  return sha256Hex(b);
}

function validateTimestampSeconds(ts) {
  if (!Number.isInteger(ts)) throw new Error('TimestampNotInteger');
  if (ts > 10_000_000_000) throw new Error('TimestampLooksLikeMilliseconds');
  if (ts < 1_577_836_800 || ts > 4_102_444_800) throw new Error('TimestampOutOfBounds');
  return true;
}

function computeFee(totalPool, feeBps) {
  if (!Number.isInteger(totalPool) || totalPool < 0) throw new Error('InvalidTotalPool');
  if (!Number.isInteger(feeBps) || feeBps < 0 || feeBps > 10_000) throw new Error('FeeBpsOutOfRange');
  return Math.floor((totalPool * feeBps) / 10_000);
}

function computePrizePool(totalPool, feeBps) {
  return totalPool - computeFee(totalPool, feeBps);
}

function computePayout(stakeAmount, prizePool, winnerPool) {
  if (!Number.isInteger(stakeAmount) || stakeAmount < 0) throw new Error('InvalidPositionAmount');
  if (!Number.isInteger(prizePool) || prizePool < 0) throw new Error('InvalidPrizePool');
  if (!Number.isInteger(winnerPool) || winnerPool < 0) throw new Error('InvalidWinnerPool');
  if (winnerPool === 0) throw new Error('DivisionByZero');
  return Math.floor((stakeAmount * prizePool) / winnerPool);
}

module.exports = {
  canonicalJson,
  computeEventIdHex,
  computeMarketIdHex,
  validateTimestampSeconds,
  computeFee,
  computePrizePool,
  computePayout,
};
