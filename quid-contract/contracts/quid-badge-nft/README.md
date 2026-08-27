# Quid Badge NFT Contract

Collectible badge NFTs for completed missions and reputation tiers, so a
hunter's track record is visible across Stellar apps.

## Features

- Bootstrap a single admin + collection metadata (`initialize`)
- Minter allow-list: the admin (reputation admin) plus any hooked contract or
  address, e.g. the `quid-store` payout hook (`add_minter` / `remove_minter`)
- Mint restricted to the admin or an allow-listed minter (`mint_badge`)
- IPFS metadata CID per badge, validated non-empty and ≤ 128 chars
- Soulbound **or** transferable, chosen per badge at mint time
- One badge per `(recipient, kind, reference)` so a retried payout hook cannot
  double mint
- Owner index for `list_by_owner` / `balance_of`
- Events: `BadgeMintEvent`, `BadgeTransferEvent`, `BadgeBurnEvent`,
  `MinterAddedEvent`, `MinterRemovedEvent`, `AdminChangedEvent`

## API

### `initialize(env, admin, name, symbol, base_uri) -> Result<(), BadgeError>`

Set the admin and collection metadata. Callable once; admin must authorize.
`name` and `symbol` must be non-empty. The admin is implicitly a minter.

### `get_admin(env) -> Result<Address, BadgeError>`

### `set_admin(env, new_admin) -> Result<(), BadgeError>`

Hand the admin role over. Current admin must authorize.

### `add_minter(env, minter) -> Result<(), BadgeError>`

Authorize an address to mint. Admin must authorize.

### `remove_minter(env, minter) -> Result<(), BadgeError>`

Revoke minting rights. Admin must authorize. The admin is never on the
allow-list and always keeps the ability to mint.

### `is_minter(env, address) -> bool`

`true` for the admin and every allow-listed minter.

### `mint_badge(env, minter, to, spec) -> Result<u64, BadgeError>`

Mint a badge to `to` and return its ID. `minter` must authorize and be the
admin or allow-listed.

| `BadgeSpec` field | Meaning |
|-------------------|---------|
| `kind` | `MissionComplete`, `ReputationTier` or `Custom` |
| `reference` | Mission ID, tier level or campaign ID, depending on `kind` |
| `metadata_cid` | IPFS CID for the badge artwork / attributes JSON |
| `soulbound` | `true` = never transferable |

### `get_badge(env, badge_id) -> Result<Badge, BadgeError>`

### `list_by_owner(env, owner) -> Vec<Badge>`

Every badge currently held by `owner`, oldest first. Capped at 256 badges per
owner so the read stays bounded.

### `list_ids_by_owner(env, owner) -> Vec<u64>`

Cheaper variant that skips the badge reads.

### `owner_of` / `metadata_cid` / `balance_of` / `badge_exists` / `total_minted`

Read helpers. `total_minted` counts mints and does not shrink on burn, since
badge IDs are monotonic.

### `has_claimed(env, owner, kind, reference) -> bool`

Whether that exact milestone was already awarded to `owner`.

### `collection` / `name` / `symbol` / `base_uri`

Collection metadata. Soroban strings cannot be concatenated on-chain, so
clients join `base_uri` with a badge's `metadata_cid` off-chain.

### `transfer(env, from, to, badge_id) -> Result<(), BadgeError>`

Move a transferable badge; `from` must authorize and own it. Soulbound badges
always fail with `SoulboundBadge`. The milestone claim stays with the original
recipient, so transferring a badge away does not free up a re-mint.

### `burn(env, caller, badge_id) -> Result<(), BadgeError>`

Destroy a badge. Callable by its owner or the admin. Releases the milestone
claim so an authorized minter can re-issue the badge later.

## Types

```text
BadgeKind  = MissionComplete | ReputationTier | Custom
BadgeSpec  { kind, reference, metadata_cid, soulbound }
Badge      { id, owner, minted_to, kind, reference, metadata_cid, soulbound,
             minted_at, minted_by }
Collection { name, symbol, base_uri }
```

`owner` is the current holder; `minted_to` is the original recipient and is
what the one-per-milestone guard is keyed on.

## Initialize after deploy

```bash
stellar contract invoke \
  --id <BADGE_NFT_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  initialize \
  --admin alice \
  --name "Quid Badges" \
  --symbol QBADGE \
  --base_uri "ipfs://"
```

Then authorize the reputation / store hook that mints on payout:

```bash
stellar contract invoke \
  --id <BADGE_NFT_CONTRACT_ID> \
  --source alice \
  --network testnet \
  -- \
  add_minter \
  --minter <STORE_OR_REPUTATION_CONTRACT_ID>
```

## Known gaps

- No `quid-store` payout → `mint_badge` cross-contract call yet; the hook is
  expected to be wired on the store side once the payout attestation lands
- No on-chain `token_uri` (Soroban strings cannot be concatenated); clients
  join `base_uri` + `metadata_cid`
- `list_by_owner` caps owners at 256 badges (`OwnerBadgeLimit`)

## Tests

```bash
cargo test -p quid-badge-nft
```
