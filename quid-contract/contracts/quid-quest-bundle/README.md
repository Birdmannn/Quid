# quid-quest-bundle

Multi-mission campaigns: a founder bundles `quid-store` mission ids under shared
metadata and escrows a bonus for each hunter who finishes the whole sequence.

Closes [#304](https://github.com/Quid-proquo/Quid/issues/304).

## Model

Three rules carry the design:

1. **A bonus is only ever paid out of escrow.** `create_bundle` pulls
   `bonus_amount * max_claims` from the founder up front, and every claim draws
   that balance down. The contract can never owe more than it holds, so
   `cancel_bundle` always refunds exactly the unclaimed remainder.

2. **Completions are reported, not asserted.** Only the configured payout hook —
   the deployed `quid-store` contract — may call `record_completion`, and only
   once per (bundle, hunter, mission). This is the same trust model
   `quid-referral` and `quid-badge-nft` use for settled payouts: a hunter cannot
   mark their own work done.

3. **The required set is frozen once the campaign pays out.** Missions can be
   added while no hunter has claimed; the first claim rejects further
   `add_mission` calls, so a finished campaign cannot be moved out of reach
   after the fact.

The bundle references mission ids only. Mission state, rewards and payouts stay
in `quid-store` — this contract layers the campaign on top and never duplicates
mission data it would then have to keep in sync.

## Double-claim prevention

| Replay | Guard |
|--------|-------|
| Same completion reported twice | `Completion(bundle, hunter, mission)` is written before the counter moves, and a second report errors with `CompletionAlreadyRecorded` — which is what keeps `CompletedCount` a count of *distinct* missions |
| Same bonus claimed twice | `claim_completion_bonus` books `Claimed(bundle, hunter)` and draws the escrow down *before* it transfers, so a repeated or re-entrant claim finds the slot taken |
| Refund taken twice | `cancel_bundle` zeroes `escrow_balance` and flips the status before transferring |

## Entrypoints

| Function | Auth | Purpose |
|----------|------|---------|
| `initialize(admin)` | `admin` | One-time bootstrap; rejects a second call |
| `get_admin()` / `set_admin(caller, new_admin)` | — / admin | Ownership |
| `set_payout_hook(caller, payout_hook)` | admin | Authorize the reporting contract |
| `get_payout_hook()` | — | Current hook |
| `create_bundle(owner, title, metadata_cid, bonus_token, bonus_amount, max_claims)` | `owner` | Open a campaign and escrow the bonus; returns the bundle id |
| `add_mission(bundle_id, mission_id)` | bundle owner | Add a `quid-store` mission to the required set; returns the new count |
| `record_completion(caller, bundle_id, hunter, mission_id)` | hook | Report one finished mission; returns the hunter's completed count |
| `claim_completion_bonus(bundle_id, hunter)` | `hunter` | Pay the bonus once every mission is done; returns the amount |
| `cancel_bundle(bundle_id)` | bundle owner | Close the campaign and refund the unclaimed escrow; returns the refund |
| `get_bundle(bundle_id)` / `get_bundle_count()` | — | Read campaign state |
| `get_mission_at(bundle_id, index)` / `is_mission_in_bundle(bundle_id, mission_id)` | — | Enumerate the required set |
| `get_completed_count(bundle_id, hunter)` / `is_completed(bundle_id, hunter, mission_id)` | — | Progress |
| `is_bundle_complete_for(bundle_id, hunter)` | — | Whether every mission is done |
| `has_claimed(bundle_id, hunter)` | — | Claim state |

## Errors

`NotAuthorized`, `AlreadyInitialized`, `NotInitialized`, `BundleNotFound`,
`InvalidState`, `InvalidAmount`, `InvalidInput`, `MissionAlreadyAdded`,
`MissionNotInBundle`, `BundleEmpty`, `CompletionAlreadyRecorded`,
`MissionsIncomplete`, `AlreadyClaimed`, `NoClaimsRemaining`,
`InsufficientEscrow`, `Overflow`.

## Build and test

```sh
cd quid-contract
cargo test -p quid-quest-bundle
stellar contract build
```
