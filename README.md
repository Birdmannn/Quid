<img width="161" height="87" alt="Quid" src="https://github.com/user-attachments/assets/70d1ce77-641c-445b-911d-971a8908593d" />

# Quid

Quid is a feedback marketplace on **Stellar / Soroban**. Founders lock USDC/XLM bounties for honest dApp feedback; hunters earn rewards for useful submissions.

## The problem

- Founders struggle to find real users to test their dApps.
- Users have little incentive to write detailed, constructive feedback.
- Discord feedback is often spam or lost in noise.

## The solution

1. **Founders** create a Mission and escrow rewards in `quid-store`.
2. **Hunters** submit feedback (text / screenshots) via IPFS; only the CID goes on-chain.
3. **Payout** happens from the smart contract when the founder approves.

### Core features

| Feature | Description |
|---------|-------------|
| **Bounty Vault** | Rewards locked on-chain when a mission is created |
| **Hybrid storage** | Feedback on IPFS; CID on Soroban |
| **Asset gating** | Optional token/NFT holdings required to submit |
| **Reputation** | On-chain attestations / profiles for quality contributors |

## Repository layout

```text
Quid/
├── frontend/          # Next.js app (Freighter, creator + hunter UI)
├── backend/           # NestJS API (auth, missions index, upload, indexer)
├── quid-contract/     # Soroban contracts (store, reputation, milestone escrow, dispute)
└── CONTRIBUTING.md
```

## Architecture

```text
Freighter wallet
      │
      ▼
Frontend (Next.js) ──────► Backend (NestJS + Postgres)
      │                         │
      │ create / submit / pay   │ index events, drafts, IPFS
      ▼                         ▼
Soroban contracts ◄─────────────┘
(quid-store, quid-reputation, quid-milestone-escrow, quid-dispute)
      │
      ▼
IPFS (feedback blobs; CID stored on-chain)
```

**Current status (honest):** Contracts are implemented and deployable. Frontend has Freighter + Horizon wiring and polished UI shells, but marketplace data is still mostly mock and contract IDs are not invoked yet. Backend has SEP-10 auth, mission reads/drafts, and stubs for IPFS upload + chain indexer. The MVP gap is wiring **create → submit → approve → payout** end-to-end.

## Tech stack

| Layer | Stack |
|-------|--------|
| Frontend | Next.js, React, TypeScript, Tailwind, shadcn/ui, Freighter, Stellar SDK |
| Backend | NestJS, Prisma, PostgreSQL, SEP-10 / JWT |
| Contracts | Rust, Soroban SDK 23, Stellar CLI |
| Payments | USDC / XLM on Stellar |

## Quick start

### Prerequisites

- Node.js 18+
- Docker (for Postgres)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools) (for contracts)
- [Freighter](https://www.freighter.app/) wallet (testnet)

### Frontend

```bash
cd frontend
npm install
cp .env.local.example .env.local   # or create .env.local (see frontend/README.md)
npm run dev
```

Open [http://localhost:3000](http://localhost:3000).

### Backend

```bash
cd backend
docker compose up -d
cp .env.example .env               # fill STELLAR_SERVER_SECRET, JWT_SECRET
npm install
npm run prisma:migrate
npm run start:dev
```

API defaults to [http://localhost:3001](http://localhost:3001).

### Contracts

```bash
cd quid-contract
stellar contract build
# deploy steps: see quid-contract/README.md
```

## Roles

### Founders (creators)

Create a mission (title, dApp URL, reward per hunter, max participants), escrow funds, review submissions, approve payouts.

### Hunters (users)

Browse the mission board, submit feedback + proof, get paid in USDC/XLM when approved.

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md). Pick issues labeled `good first issue`, `help wanted`, or `priority`.

## License

See the repository license file.
