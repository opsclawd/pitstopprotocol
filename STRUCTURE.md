# STRUCTURE.md

Spec-aligned repository structure map (source of guidance: comprehensive end-to-end spec from message 1154).

## On-chain program target
- `programs/pitstop/src/lib.rs` -> program entrypoint and module wiring
- `programs/pitstop/src/constants.rs` -> protocol constants (align with `SPEC_PROTOCOL.md` + `specs/constants.json`)
- `programs/pitstop/src/error.rs` -> protocol error enum (align with `SPEC_ERRORS.md`)
- `programs/pitstop/src/events.rs` -> event definitions (align with `SPEC_EVENTS.md`)
- `programs/pitstop/src/pda.rs` -> PDA derivation helpers (align with `SPEC_PROTOCOL.md` / `SPEC_ACCOUNTS.md`)
- `programs/pitstop/src/math.rs` -> deterministic fee/payout math (align with `SPEC_PROTOCOL.md` / unit tests)
- `programs/pitstop/src/state.rs` -> account schemas (align with `SPEC_ACCOUNTS.md`)

### Instruction modules
Each file must implement exactly one locked instruction spec from `SPEC_INSTRUCTIONS/`:
- `instructions/initialize.rs` -> `SPEC_INSTRUCTIONS/initialize.md`
- `instructions/create_market.rs` -> `SPEC_INSTRUCTIONS/create_market.md`
- `instructions/add_outcome.rs` -> `SPEC_INSTRUCTIONS/add_outcome.md`
- `instructions/finalize_seeding.rs` -> `SPEC_INSTRUCTIONS/finalize_seeding.md`
- `instructions/place_bet.rs` -> `SPEC_INSTRUCTIONS/place_bet.md`
- `instructions/lock_market.rs` -> `SPEC_INSTRUCTIONS/lock_market.md`
- `instructions/resolve_market.rs` -> `SPEC_INSTRUCTIONS/resolve_market.md`
- `instructions/void_market.rs` -> `SPEC_INSTRUCTIONS/void_market.md`
- `instructions/claim_resolved.rs` -> `SPEC_INSTRUCTIONS/claim_resolved.md`
- `instructions/claim_voided.rs` -> `SPEC_INSTRUCTIONS/claim_voided.md`
- `instructions/sweep_remaining.rs` -> `SPEC_INSTRUCTIONS/sweep_remaining.md`
- `instructions/cancel_market.rs` -> `SPEC_INSTRUCTIONS/cancel_market.md`

## Backend target (post-protocol stabilization)
- `backend/src/client/` -> program client + PDA helpers
- `backend/src/operator/` -> create/open/lock/resolve/sweep operational flows
- `backend/src/indexer/` -> account/event indexing
- `backend/src/api/` -> HTTP endpoints
- `backend/src/solana/` -> connection + signer plumbing

## Frontend target (post-protocol stabilization)
- `frontend/src/lib/` -> solana and API clients
- `frontend/src/pages/` -> market list/detail screens
- `frontend/src/components/` -> betting/claim UI components

## Test architecture
- `tests/unit/` -> pure deterministic unit tests (canonicalization/math/time)
- `tests/harness/` -> deterministic harness contracts and provider/fixture interfaces
- `tests/instructions/` -> executable instruction specs (failing-first)
- `tests/fixtures/` -> fixture contracts (USDC etc.)

## Process rule
Before instruction code changes:
1) corresponding `SPEC_INSTRUCTIONS/<instruction>.md` must be LOCKED
2) failing-first test pack for that instruction must exist
3) spec gate checks must pass
