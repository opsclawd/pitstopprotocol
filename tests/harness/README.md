# Anchor Test Harness (Issue #57)

This harness defines deterministic test contracts before protocol instruction implementation:

- `config.js` — canonical harness runtime config (RPC, WS, commitment, constants)
- `time.js` — seconds-only timestamp helpers and lock-window helpers
- `accounts.js` — assertion helpers used by integration/invariant suites
- `provider.js` — provider abstraction for local validator integration
- `../fixtures/usdc_fixture.js` — USDC fixture contract (6 decimals)
- `smoke.spec.js` — executable harness smoke test

## Deterministic timestamp strategy
- Use unix seconds only.
- Reject ms-scale timestamps.
- For lifecycle tests, derive `lock_timestamp` from deterministic helper relative to test start.

## Next step
PR #58 wires canonical + math unit tests into this harness and adds golden-vector assertions.


## Determinism controls
- Set `HARNESS_NOW_SECS` to freeze `now` in harness helpers.
- `unixNowSeconds({nowSeconds})` overrides wall-clock for deterministic tests.
- Provider exposes `deterministicSeed` for repeatable fixture derivations.

## Fixture interface
- `usdcFixtureSpec()` returns locked fixture contract shape.
- `getOrCreateUsdcMint(adapter)` delegates mint creation to adapter in #58/#59 while preserving interface stability.
