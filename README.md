# PitStop Protocol

Solana-based **parimutuel prediction markets** for motorsports — starting with an **F1 winner market MVP** built for **Solana Seeker**.

- **Network:** Solana **devnet** (MVP demo; no real money)
- **Asset:** **SOL-only** pools (MVP)
- **UX:** mobile-first, dark racing theme, swipeable driver cards, haptics
- **State updates:** real-time odds/pools via on-chain account subscriptions

## Links
- Repo: https://github.com/opsclawd/pitstopprotocol
- Project board: https://github.com/users/opsclawd/projects/1

## Tech stack (MVP)
### On-chain
- Rust + Anchor (single program)
- SOL escrow (lamports)
- Integer/fixed-point math only (no floats)

### Mobile
- Bare React Native (Android-first / Seeker-first)
- Solana Mobile (MWA / Seed Vault compatible)

### Web (stub now, real client later)
- Next.js (App Router)
- Reuses shared `packages/*`

### Oracle / automation
- Node.js + TypeScript runner
- Jolpica F1 API (schedule + post-race results)
- GitHub Actions cron initially (easy pivot to Fly.io later)

## Monorepo structure
- `apps/mobile` — bare RN app (Seeker)
- `apps/web` — Next.js stub
- `packages/core` — shared math/types
- `packages/chain` — PDAs + tx builders + subscriptions (shared)
- `packages/oracle` — settlement runner
- `docs/` — plan and architecture notes

## MVP product rules
- Winner-only markets (one race = one market)
- Users can bet **multiple times** in a market, but only **add to the same driver position** (MVP)
- Settlement is admin/oracle-triggered after race results are known

## Docs
- Project plan: `docs/PROJECT_PLAN.md`
- Architecture notes: `docs/ARCHITECTURE.md`

## Development (coming next)
Mobile app and Anchor program scaffolds will be added next. When they land, this section will include:
- wallet dev/testing (Mock MWA Wallet)
- running the mobile app on Android
- deploying program to devnet

