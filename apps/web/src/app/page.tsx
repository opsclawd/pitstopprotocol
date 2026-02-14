import { impliedPayoutMultiplier } from '@pitstopprotocol/core';

export default function Page() {
  const m = impliedPayoutMultiplier(100n, 25n);
  return (
    <main style={{ padding: 24 }}>
      <h1>PitStop Protocol (web stub)</h1>
      <p>Shared core package works. Example multiplier: {m.toFixed(2)}x</p>
    </main>
  );
}
