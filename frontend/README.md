# Quid Frontend

Next.js app for the Quid feedback marketplace: landing page, Freighter onboarding, creator and hunter dashboards.

## Stack

- Next.js (App Router) + React + TypeScript
- Tailwind CSS + shadcn/ui
- `@stellar/freighter-api` + `@stellar/stellar-sdk`
- Framer Motion (landing)

## What’s working

- Landing / marketing pages
- Freighter connect, session restore, testnet Horizon balances / Friendbot
- Role selection (creator vs hunter) via localStorage
- Creator and hunter dashboard **UI shells**
- Creator wallet page with real Horizon payments history

## What’s missing (MVP gaps)

| Area | Status |
|------|--------|
| Soroban client using `NEXT_PUBLIC_QUID_*` | Env only — not invoked |
| Create mission → on-chain escrow | Not wired |
| Hunter mission board / submissions | Placeholders or mock data |
| Submit feedback → IPFS → `submit_feedback` | Not wired |
| Approve → `payout_participant` | Stub / console only |
| Backend API integration | Not connected |
| Replace mock quest data | Still hardcoded in several features |

Priority work for contributors: wire Freighter signing to `quid-store`, then replace mocks with backend/indexer data.

## Setup

```bash
cd frontend
npm install
cp .env.local.example .env.local
# Fill contract IDs from your testnet deploy
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

### Environment

| Variable | Purpose |
|----------|---------|
| `NEXT_PUBLIC_QUID_STORE_ID` | `quid-store` contract ID |
| `NEXT_PUBLIC_QUID_REPUTATION_ID` | `quid-reputation` contract ID |
| `NEXT_PUBLIC_QUID_MILESTONE_ID` | `quid-milestone-escrow` contract ID |
| `NEXT_PUBLIC_HORIZON_URL` | Optional Horizon override |
| `NEXT_PUBLIC_FRIENDBOT_URL` | Optional Friendbot override |
| `NEXT_PUBLIC_API_URL` | Backend base URL (when integrated) |

## Scripts

```bash
npm run dev          # development server
npm run build        # production build
npm run start        # serve production build
npm run lint
npm run type-check
```

## Key routes

| Route | Description |
|-------|-------------|
| `/` | Landing |
| `/connect-wallet` | Freighter onboarding |
| `/account-type` | Creator vs hunter |
| `/creator` | Creator dashboard |
| `/creator/quests` | Quest list |
| `/creator/quests/[questId]` | Quest detail / review UI |
| `/creator/wallet` | Balances + Horizon history |
| `/hunter` | Hunter dashboard |
| `/hunter/mission-board` | Mission board (placeholder) |
| `/hunter/my-submissions` | Submissions (placeholder) |
| `/missions` | Legacy / stub board |

## Useful paths

- Wallet: `src/lib/freighter-wallet.ts`, `src/context/WalletProvider.tsx`
- Mock data: `src/features/creators/MockData.ts`, `src/app/hooks/useQuestData.ts`
- Design notes: `Design.md`

## Related docs

- Root: [../README.md](../README.md)
- Backend: [../backend/README.md](../backend/README.md)
- Contracts: [../quid-contract/README.md](../quid-contract/README.md)
