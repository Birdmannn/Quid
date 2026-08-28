# quid-fee-collector

Protocol fee vault. Receives a configurable cut of mission value, tracks the
undrawn balance per token, and lets an admin withdraw.

Closes [#295](https://github.com/Quid-proquo/Quid/issues/295).

## Model

The fee rate lives in one place — this contract — as basis points
(`1 bps = 0.01%`, `10_000 bps = 100%`). Callers pass the value the fee is
assessed on and the vault decides the cut, so nothing downstream has to be
redeployed when the rate changes.

Balances are tracked per token in contract storage rather than read from the
token contract, so a stray transfer into the vault can never be withdrawn as if
it were a collected fee.

## Entrypoints

| Function | Auth | Purpose |
|----------|------|---------|
| `initialize(admin, fee_bps)` | `admin` | One-time bootstrap; rejects a second call |
| `get_admin()` | — | Current admin |
| `set_admin(caller, new_admin)` | admin | Hand over control |
| `set_fee_bps(caller, fee_bps)` | admin | Update the rate (`<= 10_000`) |
| `get_fee_bps()` | — | Current rate |
| `compute_fee(gross_amount)` | — | Fee owed on `gross_amount`, rounded down |
| `collect_fee(from, token, gross_amount)` | `from` | Compute the cut and pull it; returns the fee taken |
| `deposit_fee(from, token, amount)` | `from` | Pull an exact amount |
| `withdraw_fees(caller, token, to, amount)` | admin | Pay out collected fees |
| `get_balance(token)` | — | Undrawn balance for one token |
| `get_balances()` | — | Undrawn balance for every token collected in |
| `get_tokens()` | — | Tokens this vault has collected in |

## Errors

| Code | Variant | When |
|------|---------|------|
| 1 | `NotAuthorized` | Caller is not the admin |
| 2 | `AlreadyInitialized` | `initialize` called twice |
| 3 | `NotInitialized` | Admin read before `initialize` |
| 4 | `InvalidFeeBps` | Rate above `10_000` |
| 5 | `InvalidAmount` | Non-positive deposit/withdrawal, or negative gross |
| 6 | `InsufficientBalance` | Withdrawal exceeds what was collected in that token |
| 7 | `Overflow` | Fee math would overflow `i128` |

## How `quid-store` uses it

`quid-store` charges the fee **once, at mission create, on top of the escrow**:

```text
owner pays  = reward_amount * max_participants  +  fee
escrow held = reward_amount * max_participants
fee         = compute_fee(escrow)
```

Charging on top rather than out of the pot means the fee never reduces a
hunter's payout, and charging once at create rather than per payout means a
mission is never billed twice for the same value. The vault itself is generic —
`deposit_fee` accepts a fee from any caller — so a payout-time fee can be added
later without touching this contract.

The wiring is opt-in: with no collector configured, `create_mission` behaves
exactly as before and charges nothing.

```bash
# point the store at the vault (see quid-contract/README.md for deploy steps)
stellar contract invoke --id <STORE_ID> --source alice --network testnet \
  -- set_fee_collector --new_collector <FEE_COLLECTOR_ID>
```

## Tests

```bash
cargo test -p quid-fee-collector   # vault: fee math, auth, withdrawal
cargo test -p quid-store           # store -> vault integration
```
