# Quid Reputation Contract

On-chain attestation and profile system for Quid contributors.

## Features

- Bootstrap a single admin (`initialize`)
- Issue / get / revoke attestations (CID-backed metadata)
- Set / get reputation profiles (`score`, `missions_completed`, `missions_created`)
- Emit `AttestationRevokedEvent` on revoke

## API

### `initialize(env, admin) -> Result<(), ReputationError>`

Set the contract admin. Callable once; admin must authorize.

### `get_admin(env) -> Result<Address, ReputationError>`

Return the admin address.

### `issue_attestation(env, issuer, subject, attestation_type, data_cid) -> Result<u64, ReputationError>`

Issue an attestation. Issuer must authorize. Returns the new attestation ID.

| Param | Meaning |
|-------|---------|
| `issuer` | Who issues the attestation |
| `subject` | Who it is about |
| `attestation_type` | Type label (string) |
| `data_cid` | IPFS CID for off-chain metadata |

### `get_attestation(env, attestation_id) -> Result<Attestation, ReputationError>`

Load an attestation by ID.

### `revoke_attestation(env, caller, attestation_id) -> Result<(), ReputationError>`

Revoke an attestation. Caller must be the issuer or admin.

### `get_attestation_count(env) -> u64`

Total attestations issued (counter).

### `attestation_exists(env, attestation_id) -> bool`

### `set_profile(env, profile) -> Result<(), ReputationError>`

Upsert a `Profile`. Subject must authorize.

### `get_profile(env, subject) -> Result<Profile, ReputationError>`

### `profile_exists(env, subject) -> bool`

## Types

```text
Attestation { id, issuer, subject, attestation_type, data_cid, issued_at, revoked }
Profile     { subject, score, missions_completed, missions_created }
```

## Initialize after deploy

```bash
stellar contract invoke \
  --id <REPUTATION_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin alice
```

## Known gaps

- No automatic `missions_completed` / score bump from `quid-store` payouts yet
- No attestation expiry field (older docs mentioned `expires_at` — not in current code)
- No issued-event (only revoke event today)

## Tests

```bash
cargo test -p quid-reputation
```
