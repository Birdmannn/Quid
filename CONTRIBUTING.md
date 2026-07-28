# Contributing to Quid

Thanks for helping build Quid — a Stellar feedback marketplace for founders and hunters.

## Why Quid

Founders launch dApps into quiet Discords. Real users rarely spend 20 minutes testing edge cases. Quid turns feedback into an escrowed transaction: founders lock rewards, hunters submit proof, payouts happen on-chain when approved.

## Repo map

| Path | What you’ll work on |
|------|---------------------|
| `frontend/` | Next.js UI, Freighter, creator/hunter flows |
| `backend/` | NestJS auth, mission index, IPFS upload, indexer |
| `quid-contract/` | Soroban contracts (`quid-store`, reputation, milestone escrow) |

Read the README in each package before starting.

## Prerequisites

- Node.js 18+
- Docker (backend Postgres)
- Rust + [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools)
- Freighter (testnet)

## Local setup

```bash
git clone https://github.com/Quid-proquo/Quid.git
cd Quid

# Frontend
cd frontend
npm install
cp .env.local.example .env.local
npm run dev

# Backend (separate terminal)
cd backend
docker compose up -d
cp .env.example .env          # set JWT_SECRET + STELLAR_SERVER_SECRET
npm install
npm run prisma:migrate
npm run start:dev

# Contracts
cd quid-contract
stellar contract build
cargo test
```

Deploy steps: see [quid-contract/README.md](./quid-contract/README.md).

## Where help is needed (MVP)

Highest impact:

1. **Frontend Soroban client** — invoke `quid-store` with Freighter (`NEXT_PUBLIC_QUID_STORE_ID`)
2. **Backend IPFS upload** — replace upload stub with real pinning + CID
3. **Backend indexer** — sync contract events into Postgres
4. **Wire dashboards** — replace mock quest data with API / chain reads
5. **Contract polish** — `reject_submission`, expiry refunds, reputation on payout

See package READMEs for detailed “what’s missing” tables.

## Finding an issue

Use labels:

- `good first issue` — small, well-scoped
- `help wanted` — needs an owner
- `priority` — MVP-critical
- `frontend` / `backend` / `contracts` / `design`

Comment on the issue before starting so work isn’t duplicated.

## Workflow

1. Branch: `feat/...`, `fix/...`, `chore/...`, `docs/...`
2. Keep PRs focused; link the issue (`Closes #123`)
3. Include screenshots for UI; include `cargo test` / `npm test` notes for logic changes
4. Open PR against `main`

## Coding standards

### Rust (contracts)

- Prefer `Result` over `.unwrap()` in contract logic
- Keep storage lean; extend TTL where the codebase already does
- Run `cargo fmt` and tests before pushing

### TypeScript (frontend / backend)

- Strict TypeScript — avoid `any`
- Frontend: functional components + Tailwind
- Backend: Nest modules, DTOs with `class-validator`

## Design contributions

See `frontend/Design.md`. Prefer issues for:

- Mission card states, create-mission wizard, submit/review flows
- Empty / loading / error states
- Mobile hunter board
- Design tokens aligned with the dark Quid brand

## Code of conduct

Be respectful. No harassment, hate speech, or spam. We’re building a trust marketplace — start in the repo.

Ready to ship.
