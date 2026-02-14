# PitStop Protocol — Project Plan (MVP)

## Synopsis
PitStop Protocol is a **Solana-based parimutuel prediction market** for motorsports (starting with **F1 winner markets**) built for **Solana Seeker**.

- **MVP (devnet / no real money):** SOL-only pools, winner-only markets, manual + automated (oracle) settlement.
- **Target users:** crypto-native motorsports fans.
- **Goal:** demo-quality mobile dApp showcasing Solana programs (Anchor), mobile wallet integration, real-time state updates, and an off-chain settler.

## MVP Scope (non-negotiables)
- Mobile-first UX (big tap targets, swipeable driver cards, dark racing theme, haptics)
- **Seamless wallet integration** (MWA / Seeker-friendly)
- **Real-time odds updates** (via on-chain account subscriptions, no backend required)
- Markets on **Solana devnet**
- **SOL-only** escrow (no SPL tokens in MVP)
- User can bet **multiple times** in the same market **for the same driver/position** (add-to-position)

## Architecture
- **On-chain:** single Anchor program implementing `create_market`, `place_bet`, `settle_market`, `claim`
- **Mobile:** bare React Native app (Android-first), Seeker UX
- **Web:** Next.js stub now; later reuse shared packages to ship a real web client
- **Oracle:** Node/TS runner (GitHub Actions cron initially, can pivot to Fly.io)

## Repo Layout
- `programs/` — Anchor program (to be added)
- `apps/mobile/` — bare React Native app (Seeker-first)
- `apps/web/` — Next.js stub
- `packages/core/` — pure TS: math/types
- `packages/chain/` — PDAs + instruction builders + account subscriptions
- `packages/oracle/` — Jolpica fetch + `settle_market` tx sender

## Roadmap (high level)
### Week 1–2: Program foundation
- Finalize account model (no floats/strings in accounts)
- Implement: create_market, place_bet, settle_market, claim
- Anchor test suite including edge cases (winner_pool=0, double-claim prevention)
- Deploy to devnet

### Week 3–4: Mobile app
- Wallet connect + sign/send transactions
- Race schedule (Jolpica)
- Market screen with driver carousel + live pools + implied payout
- Place bet flow + claim winnings + basic history
- Real-time updates by subscribing to market account

### Week 5: Oracle
- One-shot settler runner + retries/logging
- GitHub Actions cron wiring + secrets
- Manual override runbook

### Week 6: Polish + demo
- Visual system + animations + loading states
- Demo video + README + architecture writeup
- Friend testing + feedback loop

