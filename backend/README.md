# Quid Backend

NestJS API that supports Quid off-chain: wallet auth (SEP-10), mission indexing/drafts, file upload (IPFS — planned), and a Soroban event indexer (scaffold).

## Why this exists

Smart contracts hold escrow and truth for payouts. The backend makes the product usable:

- Fast mission list / search / filter (Postgres)
- Draft missions before on-chain publish
- Pin feedback media to IPFS and return CIDs
- Sync on-chain events into the DB for dashboards

The frontend should call this API for browse/auth/upload; Freighter should call contracts for create / submit / pay.

## Stack

- NestJS 11 + TypeScript
- Prisma 7 + PostgreSQL 15
- Passport JWT + Stellar SDK (SEP-10)
- `@nestjs/schedule` (indexer cron)

## What’s working

| Feature | Status |
|---------|--------|
| `GET /health` | Live |
| SEP-10 challenge + verify → JWT | Live |
| Mission list / detail / `me` / submissions (read) | Live |
| Mission drafts (`POST /missions/drafts`) | Live |
| Upload endpoints | Stub (acks bytes/JSON only — no IPFS) |
| Chain indexer cron | Scaffold (checkpoint only — no event sync) |

## What’s missing (MVP gaps)

- Real IPFS / Pinata (or similar) pinning → return CID
- Indexer: poll `quid-store` events → upsert missions/submissions
- Publish mission after on-chain create (persist `onChainId`)
- Create submission / approve / reject API flows synced with chain
- Hardening: rate limits, locked-down CORS, production secrets
- Frontend not wired to this API yet

## Setup

### Prerequisites

- Node.js 18+
- Docker (Postgres)

### Database

```bash
cd backend
docker compose up -d
```

Starts Postgres on `localhost:5432` (`quid` / `quid` / `quid_dev`).

### Env

```bash
cp .env.example .env
```

| Variable | Purpose |
|----------|---------|
| `DATABASE_URL` | Postgres connection string |
| `PORT` | API port (default `3001` in `.env.example`) |
| `JWT_SECRET` | JWT signing secret |
| `STELLAR_SERVER_SECRET` | Server keypair secret for SEP-10 |
| `HOME_DOMAIN` / `WEB_AUTH_DOMAIN` | SEP-10 domains |
| `STELLAR_NETWORK` | Network passphrase (testnet by default) |
| `RPC_URL` / `CONTRACT_ID` | For indexer (when implemented) |

Generate a Stellar keypair for `STELLAR_SERVER_SECRET` (keep it server-side only).

### Install & run

```bash
npm install
npm run prisma:migrate
npm run start:dev
```

API: [http://localhost:3001](http://localhost:3001)

## HTTP surface

| Method | Path | Auth | Notes |
|--------|------|------|-------|
| `GET` | `/health` | No | Health check |
| `GET` | `/auth/challenge?address=` | No | SEP-10 challenge |
| `POST` | `/auth/verify` | No | Returns JWT |
| `GET` | `/missions` | No | List / filter |
| `GET` | `/missions/me` | JWT | Caller’s missions |
| `GET` | `/missions/:id` | No | Detail |
| `GET` | `/missions/:id/submissions` | JWT | Owner only |
| `POST` | `/missions/drafts` | JWT | Save draft |
| `POST` | `/upload` | JWT | Stub |
| `POST` | `/upload/json` | JWT | Stub |

## Scripts

```bash
npm run start:dev       # watch mode
npm run build
npm run start:prod
npm run test
npm run test:e2e
npm run prisma:generate
npm run prisma:migrate
npm run prisma:studio
```

## Project layout

```text
backend/
├── docker-compose.yml
├── prisma/                 # schema + migrations
└── src/
    ├── auth/               # SEP-10 + JWT
    ├── missions/
    ├── upload/             # stub
    ├── indexer/            # scaffold
    └── prisma/
```

## Related docs

- Root: [../README.md](../README.md)
- Frontend: [../frontend/README.md](../frontend/README.md)
- Contracts: [../quid-contract/README.md](../quid-contract/README.md)
