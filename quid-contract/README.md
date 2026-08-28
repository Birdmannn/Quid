# Quid Contracts

Soroban (Rust) smart contracts for Quid: bounty escrow, reputation, and milestone programs.

## Contracts

| Package | Wasm | Role |
|---------|------|------|
| `quid-store` | `quid_store.wasm` | Mission bounty vault: create, submit, payout, cancel, pause, slash |
| `quid-reputation` | `quid_reputation.wasm` | Admin, profiles, attestations |
| `quid-milestone-escrow` | `quid_milestone_escrow.wasm` | Multi-milestone escrow programs |
| `quid-referral` | `quid_referral.wasm` | Referral attribution + rewards on settled payouts |
| `hello-world` | `hello_world.wasm` | Scaffold only — safe to ignore |

## Prerequisites

- Rust (stable)
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools) (`stellar`) — use a version compatible with Soroban SDK 23
- Testnet account (Friendbot)

```bash
stellar --version
stellar keys generate alice --network testnet --as-secret
stellar keys fund alice --network testnet
```

## Build

```bash
cd quid-contract
stellar contract build
```

Wasm output:

```text
target/wasm32v1-none/release/quid_store.wasm
target/wasm32v1-none/release/quid_reputation.wasm
target/wasm32v1-none/release/quid_milestone_escrow.wasm
target/wasm32v1-none/release/quid_referral.wasm
```

## Deploy (testnet)

```bash
stellar contract deploy \
  --wasm target/wasm32v1-none/release/quid_store.wasm \
  --source alice \
  --network testnet

stellar contract deploy \
  --wasm target/wasm32v1-none/release/quid_reputation.wasm \
  --source alice \
  --network testnet

stellar contract deploy \
  --wasm target/wasm32v1-none/release/quid_milestone_escrow.wasm \
  --source alice \
  --network testnet

stellar contract deploy \
  --wasm target/wasm32v1-none/release/quid_referral.wasm \
  --source alice \
  --network testnet
```

Copy each `C...` contract ID into `frontend/.env.local`.

### Initialize reputation (once)

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin alice
```

Verify:

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  get_admin
```

### Initialize referrals (once)

```bash
# 500 bps = 5% of each referred hunter's payout
stellar contract invoke \
  --id <REFERRAL_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin alice \
  --reward_bps 500

# only this address may report payouts
stellar contract invoke \
  --id <REFERRAL_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  set_payout_hook \
  --caller alice \
  --payout_hook <STORE_CONTRACT_ID>
```

## Main entrypoints

### `quid-store`

- `create_mission` — escrow rewards, optional asset gate
- `submit_feedback` — hunter stake + IPFS CID
- `payout_participant` — pay hunter, refund stake
- `cancel_mission` / `pause_mission` / `update_mission_status`
- `slash_hunter_stake` / treasury helpers

### `quid-reputation`

- `initialize` / `get_admin`
- `issue_attestation` / `get_attestation` / `revoke_attestation`
- `set_profile` / `get_profile`

See [contracts/quid-reputation/README.md](./contracts/quid-reputation/README.md).

### `quid-referral`

- `initialize` / `set_reward_bps` / `compute_reward`
- `register_referral` / `get_referral` / `get_referred_count`
- `set_payout_hook` / `record_payout` — accrual gated behind a settled payout
- `fund` / `claim_reward` / `get_claimable` / `get_claimed`

See [contracts/quid-referral/README.md](./contracts/quid-referral/README.md).

### `quid-milestone-escrow`

- `create_program` / `add_milestone` / `approve_milestone` / `cancel_program`
- getters for program / milestone status

## Tests

```bash
cargo test
# or per package:
cargo test -p quid-store
cargo test -p quid-reputation
cargo test -p quid-milestone-escrow
cargo test -p quid-referral
```

## Known gaps (good contributor targets)

- `reject_submission` + stake refund on `quid-store`
- Mission expiry / auto-refund
- Store → reputation hook on successful payout (also wires `quid-referral.record_payout`)
- Align milestone status helpers with production auth rules
- Remove or archive `hello-world`

## Workspace layout

```text
quid-contract/
├── Cargo.toml                 # workspace (soroban-sdk 23)
└── contracts/
    ├── quid-store/
    ├── quid-reputation/
    ├── quid-milestone-escrow/
    ├── quid-referral/
    └── hello-world/
```

## Related docs

- Root: [../README.md](../README.md)
- Frontend env: [../frontend/README.md](../frontend/README.md)
- Contributing: [../CONTRIBUTING.md](../CONTRIBUTING.md)
