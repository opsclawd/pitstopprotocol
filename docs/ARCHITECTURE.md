# Architecture Notes

## Canonical reference implementations
We use Solana Mobile's official React Native samples as reference patterns:
- https://github.com/solana-mobile/react-native-samples

Most relevant sample patterns:
- **Cause Pots**: Anchor program integration, PDAs, custom instructions
- **Settle**: wallet connect UX and transaction flows

## Guiding principles
- Keep the program deterministic and minimal (single program MVP).
- Keep driver metadata off-chain (names/photos/etc.), only store pool totals + winner index on-chain.
- Real-time odds via websocket subscription to the Market account.
- Oracle is an off-chain automation that submits settlement transactions; it is not a smart contract.

## Wallet portability strategy
- `packages/chain` defines a small wallet interface.
- `apps/mobile` implements it using Mobile Wallet Adapter (Seeker-compatible).
- `apps/web` implements it using standard web wallet adapters later.


## Program design docs
- Account model + constants: `docs/program/account-model.md`
