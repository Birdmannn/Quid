# E2E Happy Path: Founder → Hunter → Payout

> Issue #343 – Manual + scripted checklist proving the full Quid flow on Stellar testnet.

---

## Prerequisites

| Requirement | Where to get it |
|-------------|----------------|
| Node.js ≥ 20 | https://nodejs.org |
| Freighter browser extension | https://www.freighter.app/ |
| Two testnet Stellar wallets | Freighter → network: Testnet |
| Testnet XLM (Founder wallet) | https://laboratory.stellar.org/#account-creator or `scripts/friendbot.sh` |
| Testnet XLM (Hunter wallet) | Same as above |
| Running backend (`PORT=3001`) | `cd backend && npm run start:dev` |
| Running frontend (`PORT=3000`) | `cd frontend && npm run dev` |

---

## Wallet Setup

```bash
# 1. Fund both wallets via Friendbot
FOUNDER=G...<founder public key>
HUNTER=G...<hunter public key>

curl "https://friendbot.stellar.org?addr=$FOUNDER"
curl "https://friendbot.stellar.org?addr=$HUNTER"
```

Verify balance ≥ 10 000 XLM on each wallet via Stellar Expert or Horizon:

```bash
curl "https://horizon-testnet.stellar.org/accounts/$FOUNDER" | jq '.balances'
```

---

## Step-by-step Checklist

### 1  Founder: Create a Mission (Escrow)

- [ ] Open http://localhost:3000 in Chrome with Freighter (Testnet).
- [ ] Connect **Founder** wallet.
- [ ] Navigate to **Creator Dashboard → New Quest**.
- [ ] Fill in title, description, reward token (XLM), reward amount (e.g. 100 XLM), max participants.
- [ ] Click **Create Quest** → approve the Soroban `create_mission` transaction in Freighter.
- [ ] Confirm: the mission appears in **Creator Dashboard → My Quests** with status `OPEN`.
- [ ] Note the **mission ID** from the URL or API response.

**Backend verification:**
```bash
curl http://localhost:3001/missions | jq '[.[] | select(.status=="OPEN")]'
```

Expected: the new mission appears in the list.

---

### 2  Hunter: Discover and Submit Feedback

- [ ] Switch Freighter to **Hunter** wallet.
- [ ] Open http://localhost:3000 in a new browser session / incognito.
- [ ] Connect Hunter wallet, complete onboarding (select **Hunter** role).
- [ ] Navigate to **Hunter Dashboard → All Quest** — the new mission appears.
- [ ] Click **Submit Feedback** on the mission.
- [ ] Enter detailed feedback text and optionally a proof URL.
- [ ] Click **Upload & Submit On-Chain** → wait for IPFS upload step.
- [ ] Approve the Soroban `submit_feedback` transaction in Freighter.
- [ ] Confirm: the **Pending** badge appears on the quest row immediately after sign.
- [ ] Wait ≤ 30 s — the Pending badge should clear once the indexer picks up the event.
- [ ] The quest now shows under **My Quest** tab with status `Submitted`.

**Indexer lag SLA**: The pending badge must disappear within 30 seconds of the transaction landing on-chain (5-second polling interval × at most 6 cycles).

**Backend verification:**
```bash
MISSION_ID=<uuid from step 1>
FOUNDER_TOKEN=<JWT from Founder SEP-10 auth>
curl -H "Authorization: Bearer $FOUNDER_TOKEN" \
  http://localhost:3001/missions/$MISSION_ID/submissions | jq '.'
```

Expected: one submission with `status: "PENDING"`, `hunterAddress` matching Hunter wallet.

---

### 3  Founder: Approve Submission and Trigger Payout

- [ ] Switch back to Founder wallet / session.
- [ ] Open **Creator Dashboard → My Quests → [mission title] → Submissions**.
- [ ] The hunter's submission is visible with status `Pending`.
- [ ] Click **Approve** → approve the Soroban `payout_participant` transaction in Freighter.
- [ ] Confirm: submission status changes to `Approved` / `Paid` in the UI.

**Backend verification:**
```bash
curl -X POST -H "Authorization: Bearer $FOUNDER_TOKEN" \
  http://localhost:3001/missions/$MISSION_ID/submissions/$SUBMISSION_ID/approve
```

Expected: `{ "status": "APPROVED" }`.

---

### 4  Dashboard Reflects Final Statuses

- [ ] Founder dashboard: mission participant count incremented.
- [ ] Hunter dashboard: quest appears with status `Submitted` / `Approved`.
- [ ] Hunter Stellar balance increased by the reward amount (check Horizon or Freighter balance).

**Hunter balance verification:**
```bash
curl "https://horizon-testnet.stellar.org/accounts/$HUNTER" | jq '.balances'
```

Expected: XLM balance increased by the reward amount minus any fees.

---

## Indexer Lag SLA

| Event | Expected badge duration |
|-------|------------------------|
| Tx signed (submit_feedback) | Pending badge shown immediately |
| Indexer catches up | Badge clears within 30 s |
| Page refresh before indexer | Badge re-appears (from localStorage) |
| 30-min timeout (no indexer) | Entry pruned, badge removed |

---

## Acceptance Criteria Status

| Criterion | How verified |
|-----------|-------------|
| Escrow → submit → payout complete | Steps 1–3 above |
| Hunter balance increases | Step 4, Horizon API |
| Dashboards reflect statuses | Step 4, UI + API checks |

---

## Automated Test Script

See [`scripts/e2e-happy-path.sh`](../../scripts/e2e-happy-path.sh) for a non-interactive
curl-based smoke test that can run against a live staging environment.
