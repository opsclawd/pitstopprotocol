export type OutcomeIndex = number;

export function impliedPayoutMultiplier(totalPool: bigint, outcomePool: bigint): number {
  // MVP UI helper (off-chain): multiplier ~= totalPool / outcomePool
  if (outcomePool === 0n) return Infinity;
  return Number(totalPool) / Number(outcomePool);
}
