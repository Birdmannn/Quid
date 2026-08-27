# Quid Contracts

Soroban (Rust) smart contracts for Quid: bounty escrow, reputation, milestone programs, and disputes.

## Contracts

| Package | Wasm | Role |
|---------|------|------|
| `quid-store` | `quid_store.wasm` | Mission bounty vault: create, submit, payout, cancel, pause, slash |
| `quid-reputation` | `quid_reputation.wasm` | Admin, profiles, attestations |
| `quid-milestone-escrow` | `quid_milestone_escrow.wasm` | Multi-milestone escrow programs |
| `quid-dispute` | `quid_dispute.wasm` | Contested-submission arbitration: timelock + optional arbiter |
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
target/wasm32v1-none/release/quid_dispute.wasm
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
  --wasm target/wasm32v1-none/release/quid_dispute.wasm \
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

### `quid-milestone-escrow`

- `create_program` / `add_milestone` / `approve_milestone` / `cancel_program`
- getters for program / milestone status

### `quid-dispute`

- `create_dispute` — hunter opens a case, stakes a bond, sets timelock + optional arbiter
- `stake_bond` — hunter adds to their bond, or respondent posts a counter-bond
- `resolve_by_arbiter` — named arbiter awards the pot before the deadline
- `timeout_release` — after the deadline, refund each party's bond
- Events: `DisputeCreatedEvent`, `BondStakedEvent`, `DisputeResolvedEvent`, `DisputeTimeoutEvent`

## Tests

```bash
cargo test
# or per package:
cargo test -p quid-store
cargo test -p quid-reputation
cargo test -p quid-milestone-escrow
cargo test -p quid-dispute
```

## Known gaps (good contributor targets)

- `reject_submission` + stake refund on `quid-store`
- Mission expiry / auto-refund
- Store → reputation hook on successful payout
- Align milestone status helpers with production auth rules
- Wire `quid-dispute` into store reject / payout holds
- Remove or archive `hello-world`

## Workspace layout

```text
quid-contract/
├── Cargo.toml                 # workspace (soroban-sdk 23)
└── contracts/
    ├── quid-store/
    ├── quid-reputation/
    ├── quid-milestone-escrow/
    ├── quid-dispute/
    └── hello-world/
```

## Related docs

- Root: [../README.md](../README.md)
- Frontend env: [../frontend/README.md](../frontend/README.md)
- Contributing: [../CONTRIBUTING.md](../CONTRIBUTING.md)
