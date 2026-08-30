#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, Address, Env, String, Vec};

mod error;
mod types;

pub use error::BadgeError;
pub use types::{Badge, BadgeKind, BadgeSpec, Collection};

use types::DataKey;

/// ~60 days of ledgers, matching the other Quid contracts.
const BADGE_TTL_LEDGERS: u32 = 5_184_000;

/// A base32 CIDv1 is 59 chars; this leaves headroom without allowing
/// unbounded metadata to be written into ledger state.
const MAX_CID_LEN: u32 = 128;

/// Caps the per-owner index so `list_by_owner` stays bounded.
const MAX_BADGES_PER_OWNER: u32 = 256;

#[contractevent(topics = ["badge", "mint"])]
pub struct BadgeMintEvent {
    pub badge_id: u64,
    pub to: Address,
    pub minter: Address,
}

#[contractevent(topics = ["badge", "transfer"])]
pub struct BadgeTransferEvent {
    pub badge_id: u64,
    pub from: Address,
    pub to: Address,
}

#[contractevent(topics = ["badge", "burn"])]
pub struct BadgeBurnEvent {
    pub badge_id: u64,
    pub owner: Address,
}

#[contractevent(topics = ["minter", "added"], data_format = "single-value")]
pub struct MinterAddedEvent {
    pub minter: Address,
}

#[contractevent(topics = ["minter", "removed"], data_format = "single-value")]
pub struct MinterRemovedEvent {
    pub minter: Address,
}

#[contractevent(topics = ["admin", "changed"])]
pub struct AdminChangedEvent {
    pub previous_admin: Address,
    pub new_admin: Address,
}

/// Badge NFTs for completed missions and reputation tiers.
///
/// Minting is restricted to an allow-list of minters (the reputation admin and
/// the `quid-store` payout hook), each badge carries an IPFS metadata CID, and
/// badges may be minted soulbound (never transferable) or transferable.
#[contract]
pub struct QuidBadgeNftContract;

#[contractimpl]
impl QuidBadgeNftContract {
    // -------------------------------------------------------------------------
    // Admin bootstrap
    // -------------------------------------------------------------------------

    /// Set the admin and collection metadata. Callable once; admin must authorize.
    /// The admin is implicitly a minter and manages the minter allow-list.
    pub fn initialize(
        env: Env,
        admin: Address,
        name: String,
        symbol: String,
        base_uri: String,
    ) -> Result<(), BadgeError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(BadgeError::AlreadyInitialized);
        }

        if name.is_empty() || symbol.is_empty() {
            return Err(BadgeError::InvalidCollection);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(
            &DataKey::Collection,
            &Collection {
                name,
                symbol,
                base_uri,
            },
        );

        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, BadgeError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(BadgeError::NotInitialized)
    }

    /// Hand the admin role to another address. Current admin must authorize.
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), BadgeError> {
        let previous_admin = Self::get_admin(env.clone())?;
        previous_admin.require_auth();

        env.storage().instance().set(&DataKey::Admin, &new_admin);

        AdminChangedEvent {
            previous_admin,
            new_admin,
        }
        .publish(&env);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Collection metadata (SEP-41 style getters)
    // -------------------------------------------------------------------------

    pub fn collection(env: Env) -> Result<Collection, BadgeError> {
        env.storage()
            .instance()
            .get(&DataKey::Collection)
            .ok_or(BadgeError::NotInitialized)
    }

    pub fn name(env: Env) -> Result<String, BadgeError> {
        Ok(Self::collection(env)?.name)
    }

    pub fn symbol(env: Env) -> Result<String, BadgeError> {
        Ok(Self::collection(env)?.symbol)
    }

    /// Gateway prefix to join with a badge's `metadata_cid` off-chain.
    pub fn base_uri(env: Env) -> Result<String, BadgeError> {
        Ok(Self::collection(env)?.base_uri)
    }

    // -------------------------------------------------------------------------
    // Minter allow-list
    // -------------------------------------------------------------------------

    /// Authorize an address to mint badges (e.g. the `quid-store` payout hook).
    /// Admin must authorize.
    pub fn add_minter(env: Env, minter: Address) -> Result<(), BadgeError> {
        let admin = Self::get_admin(env.clone())?;
        admin.require_auth();

        let key = DataKey::Minter(minter.clone());
        if env.storage().persistent().has(&key) {
            return Err(BadgeError::AlreadyMinter);
        }

        env.storage().persistent().set(&key, &true);
        env.storage()
            .persistent()
            .extend_ttl(&key, BADGE_TTL_LEDGERS, BADGE_TTL_LEDGERS);

        MinterAddedEvent { minter }.publish(&env);

        Ok(())
    }

    /// Revoke minting rights. Admin must authorize. The admin itself always
    /// keeps the ability to mint and is not stored on the allow-list.
    pub fn remove_minter(env: Env, minter: Address) -> Result<(), BadgeError> {
        let admin = Self::get_admin(env.clone())?;
        admin.require_auth();

        let key = DataKey::Minter(minter.clone());
        if !env.storage().persistent().has(&key) {
            return Err(BadgeError::MinterNotFound);
        }

        env.storage().persistent().remove(&key);

        MinterRemovedEvent { minter }.publish(&env);

        Ok(())
    }

    /// `true` for the admin and for every allow-listed minter.
    pub fn is_minter(env: Env, address: Address) -> bool {
        match env.storage().instance().get::<_, Address>(&DataKey::Admin) {
            Some(admin) if admin == address => true,
            Some(_) => env.storage().persistent().has(&DataKey::Minter(address)),
            None => false,
        }
    }

    // -------------------------------------------------------------------------
    // Minting
    // -------------------------------------------------------------------------

    /// Mint a badge to `to`. Only the admin or an allow-listed minter may call,
    /// and each recipient can hold at most one badge per `(kind, reference)`
    /// pair so a retried payout hook cannot double mint.
    pub fn mint_badge(
        env: Env,
        minter: Address,
        to: Address,
        spec: BadgeSpec,
    ) -> Result<u64, BadgeError> {
        minter.require_auth();
        Self::require_minter(&env, &minter)?;
        Self::validate_cid(&spec.metadata_cid)?;

        let claim_key = DataKey::Claimed(to.clone(), spec.kind, spec.reference);
        if env.storage().persistent().has(&claim_key) {
            return Err(BadgeError::AlreadyMinted);
        }

        let badge_id = Self::get_next_badge_id(&env);

        let badge = Badge {
            id: badge_id,
            owner: to.clone(),
            minted_to: to.clone(),
            kind: spec.kind,
            reference: spec.reference,
            metadata_cid: spec.metadata_cid,
            soulbound: spec.soulbound,
            minted_at: env.ledger().timestamp(),
            minted_by: minter.clone(),
        };

        Self::write_badge(&env, &badge);
        Self::index_add(&env, &to, badge_id)?;

        env.storage().persistent().set(&claim_key, &badge_id);
        env.storage()
            .persistent()
            .extend_ttl(&claim_key, BADGE_TTL_LEDGERS, BADGE_TTL_LEDGERS);

        BadgeMintEvent {
            badge_id,
            to,
            minter,
        }
        .publish(&env);

        Ok(badge_id)
    }

    // -------------------------------------------------------------------------
    // Reads
    // -------------------------------------------------------------------------

    pub fn get_badge(env: Env, badge_id: u64) -> Result<Badge, BadgeError> {
        env.storage()
            .persistent()
            .get(&DataKey::Badge(badge_id))
            .ok_or(BadgeError::BadgeNotFound)
    }

    /// Every badge currently held by `owner`, oldest first.
    pub fn list_by_owner(env: Env, owner: Address) -> Vec<Badge> {
        let mut badges = Vec::new(&env);

        for badge_id in Self::owned_ids(&env, &owner).iter() {
            if let Some(badge) = env
                .storage()
                .persistent()
                .get::<_, Badge>(&DataKey::Badge(badge_id))
            {
                badges.push_back(badge);
            }
        }

        badges
    }

    /// Cheaper variant of [`Self::list_by_owner`] that skips the badge reads.
    pub fn list_ids_by_owner(env: Env, owner: Address) -> Vec<u64> {
        Self::owned_ids(&env, &owner)
    }

    pub fn owner_of(env: Env, badge_id: u64) -> Result<Address, BadgeError> {
        Ok(Self::get_badge(env, badge_id)?.owner)
    }

    pub fn metadata_cid(env: Env, badge_id: u64) -> Result<String, BadgeError> {
        Ok(Self::get_badge(env, badge_id)?.metadata_cid)
    }

    pub fn balance_of(env: Env, owner: Address) -> u32 {
        Self::owned_ids(&env, &owner).len()
    }

    /// Badge ids are monotonic, so this counts mints and does not shrink on burn.
    pub fn total_minted(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::BadgeCount)
            .unwrap_or(0)
    }

    pub fn badge_exists(env: Env, badge_id: u64) -> bool {
        env.storage().persistent().has(&DataKey::Badge(badge_id))
    }

    /// Whether `owner` was already awarded this exact milestone.
    pub fn has_claimed(env: Env, owner: Address, kind: BadgeKind, reference: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Claimed(owner, kind, reference))
    }

    // -------------------------------------------------------------------------
    // Transfer / burn
    // -------------------------------------------------------------------------

    /// Move a transferable badge. Soulbound badges always fail with
    /// [`BadgeError::SoulboundBadge`]. The one-per-milestone guard stays with
    /// the original recipient, so transferring does not free up a re-mint.
    pub fn transfer(env: Env, from: Address, to: Address, badge_id: u64) -> Result<(), BadgeError> {
        from.require_auth();

        let mut badge = Self::get_badge(env.clone(), badge_id)?;

        if badge.owner != from {
            return Err(BadgeError::NotBadgeOwner);
        }
        if badge.soulbound {
            return Err(BadgeError::SoulboundBadge);
        }
        if from == to {
            return Err(BadgeError::SelfTransfer);
        }

        Self::index_add(&env, &to, badge_id)?;
        Self::index_remove(&env, &from, badge_id);

        badge.owner = to.clone();
        Self::write_badge(&env, &badge);

        BadgeTransferEvent { badge_id, from, to }.publish(&env);

        Ok(())
    }

    /// Destroy a badge. Callable by its owner or the admin. The milestone claim
    /// is released so an authorized minter can re-issue the badge later.
    pub fn burn(env: Env, caller: Address, badge_id: u64) -> Result<(), BadgeError> {
        caller.require_auth();

        let badge = Self::get_badge(env.clone(), badge_id)?;
        let admin = Self::get_admin(env.clone())?;

        if caller != badge.owner && caller != admin {
            return Err(BadgeError::NotAuthorized);
        }

        env.storage().persistent().remove(&DataKey::Badge(badge_id));
        Self::index_remove(&env, &badge.owner, badge_id);
        env.storage().persistent().remove(&DataKey::Claimed(
            badge.minted_to,
            badge.kind,
            badge.reference,
        ));

        BadgeBurnEvent {
            badge_id,
            owner: badge.owner,
        }
        .publish(&env);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Internal helpers
    // -------------------------------------------------------------------------

    fn require_minter(env: &Env, caller: &Address) -> Result<(), BadgeError> {
        let admin = Self::get_admin(env.clone())?;

        if *caller == admin {
            return Ok(());
        }

        if env
            .storage()
            .persistent()
            .has(&DataKey::Minter(caller.clone()))
        {
            return Ok(());
        }

        Err(BadgeError::NotAuthorized)
    }

    fn validate_cid(metadata_cid: &String) -> Result<(), BadgeError> {
        if metadata_cid.is_empty() || metadata_cid.len() > MAX_CID_LEN {
            return Err(BadgeError::InvalidMetadata);
        }
        Ok(())
    }

    fn get_next_badge_id(env: &Env) -> u64 {
        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::BadgeCount)
            .unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::BadgeCount, &count);
        count
    }

    fn write_badge(env: &Env, badge: &Badge) {
        let key = DataKey::Badge(badge.id);
        env.storage().persistent().set(&key, badge);
        env.storage()
            .persistent()
            .extend_ttl(&key, BADGE_TTL_LEDGERS, BADGE_TTL_LEDGERS);
    }

    fn owned_ids(env: &Env, owner: &Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::Owned(owner.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Append `badge_id` to an owner's index. Returning an error reverts the
    /// whole invocation, so a rejected mint leaves no partial state behind.
    fn index_add(env: &Env, owner: &Address, badge_id: u64) -> Result<(), BadgeError> {
        let mut owned = Self::owned_ids(env, owner);

        if owned.len() >= MAX_BADGES_PER_OWNER {
            return Err(BadgeError::OwnerBadgeLimit);
        }

        owned.push_back(badge_id);
        Self::write_owned(env, owner, &owned);

        Ok(())
    }

    fn index_remove(env: &Env, owner: &Address, badge_id: u64) {
        let owned = Self::owned_ids(env, owner);
        let mut remaining = Vec::new(env);

        for id in owned.iter() {
            if id != badge_id {
                remaining.push_back(id);
            }
        }

        Self::write_owned(env, owner, &remaining);
    }

    fn write_owned(env: &Env, owner: &Address, owned: &Vec<u64>) {
        let key = DataKey::Owned(owner.clone());
        env.storage().persistent().set(&key, owned);
        env.storage()
            .persistent()
            .extend_ttl(&key, BADGE_TTL_LEDGERS, BADGE_TTL_LEDGERS);
    }
}

mod test;
