# quid-referral

On-chain referral rewards: a referrer earns a cut when a hunter they brought in
completes a paid mission.

Closes [#301](https://github.com/Quid-proquo/Quid/issues/301).

## Model

Three rules carry the design:

1. **A hunter has exactly one referrer, for life.** The referral record is keyed
   by the referred address, so attribution can never be overwritten or contested.
   The referred hunter authorizes the registration, so nobody can be claimed as
   someone's referral against their will.

2. **Rewards accrue only from a settled payout.** `record_payout` is the single
   path that grows a claimable balance, and only the configured payout hook —
   the deployed `quid-store` contract — may call it. There is no way to claim
   speculatively against a mission that has not paid out.

3. **Rewards are paid from a funded pool, never minted.** `fund` tops the pool
   up; `claim_reward` refuses rather than half-paying if the pool is short, and
   the balance stays claimable until it can settle in full.

The referral link lives off-chain. `link_cid` points at it and `link_hash` is
the on-chain SHA-256 commitment to that CID, so an indexer can match a record to
its blob without trusting the string it was handed.

## Double-claim prevention

Two distinct replays are blocked, at the storage layer rather than in
application logic:

| Replay | Guard |
|--------|-------|
| Same payout reported twice | `PayoutRecorded(mission_id, referred)` is written before any accrual, and a second report errors with `PayoutAlreadyRecorded` — including when the token is swapped, since the key does not include it |
| Same balance claimed twice | `claim_reward` zeroes the claimable balance before it transfers, so a re-entrant or repeated claim finds nothing and errors with `NothingToClaim` |

The dedupe slot is claimed even when the hunter has no referrer, so a referral
registered after a payout cannot be back-paid for it.

## Entrypoints

| Function | Auth | Purpose |
|----------|------|---------|
| `initialize(admin, reward_bps)` | `admin` | One-time bootstrap; rejects a second call |
| `get_admin()` / `set_admin(caller, new_admin)` | — / admin | Ownership |
| `set_reward_bps(caller, reward_bps)` | admin | Update the rate (`<= 10_000`) |
| `get_reward_bps()` / `compute_reward(payout_amount)` | — | Current rate; reward owed, rounded down |
| `set_payout_hook(caller, payout_hook)` | admin | Authorize the reporting contract |
| `get_payout_hook()` | — | Current hook |
| `register_referral(referrer, referred, link_cid)` | `referred` | Record attribution + link commitment |
| `get_referral(referred)` / `has_referral(referred)` | — | Read a record |
| `get_referred_count(referrer)` | — | How many hunters an address referred |
| `record_payout(caller, referred, mission_id, token, payout_amount)` | hook | Accrue the referrer's cut; returns the amount (0 if unreferred) |
| `is_payout_recorded(mission_id, referred)` | — | Dedupe state |
| `fund(from, token, amount)` | `from` | Top up the reward pool |
| `get_pool_balance(token)` | — | Pool balance |
| `claim_reward(referrer, token)` | `referrer` | Pay out everything accrued; returns the amount |
| `get_claimable` / `get_claimed` | — | Per-referrer, per-token balances |

## Errors

| Code | Variant | When |
|------|---------|------|
| 1 | `NotAuthorized` | Caller is not the admin, or not the payout hook |
| 2 | `AlreadyInitialized` | `initialize` called twice |
| 3 | `NotInitialized` | Admin read before `initialize` |
| 4 | `InvalidRewardBps` | Rate above `10_000` |
| 5 | `InvalidAmount` | Non-positive payout/funding, or negative amount |
| 6 | `InvalidInput` | Empty `link_cid` |
| 7 | `SelfReferral` | `referrer == referred` |
| 8 | `AlreadyRegistered` | Hunter already has a referrer |
| 9 | `ReferralNotFound` | No record for that hunter |
| 10 | `PayoutAlreadyRecorded` | This (mission, hunter) payout was already accrued |
| 11 | `NothingToClaim` | Claimable balance is zero |
| 12 | `InsufficientFunds` | Pool cannot cover the claim |
| 13 | `Overflow` | Reward math would overflow `i128` |

## Wiring the payout hook

`quid-store` calling `record_payout` on successful payout is tracked separately
in [#290](https://github.com/Quid-proquo/Quid/issues/290). This crate ships the
receiving half so the two can land independently: `record_payout` is written to
be safe to call for every payout (it returns `0` for unreferred hunters instead
of failing), and `src/test.rs` exercises it from a stand-in contract to prove
the cross-contract shape works.

```bash
stellar contract invoke --id <REFERRAL_ID> --source alice --network testnet \
  -- set_payout_hook --caller alice --payout_hook <STORE_ID>
```

## Tests

```bash
cargo test -p quid-referral
```

Coverage includes double-claim and double-record prevention, hook
authorization, self-referral and re-registration, rate-change auth, pool
underfunding, and per-token accrual isolation.
