use soroban_sdk::{contracttype, Address, String};

/// What a badge was awarded for. `reference` on the badge disambiguates the
/// individual award inside a kind.
#[derive(Clone, Debug, Default, PartialEq, Eq, Copy)]
#[contracttype]
pub enum BadgeKind {
    /// Awarded on a paid-out mission submission. `reference` = mission id.
    #[default]
    MissionComplete,
    /// Awarded when a hunter crosses a reputation tier. `reference` = tier level.
    ReputationTier,
    /// Anything else the reputation admin wants to hand out.
    /// `reference` = campaign / season id.
    Custom,
}

/// Collection level metadata, set once at initialization.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Collection {
    pub name: String,
    pub symbol: String,
    /// Gateway prefix for `Badge::metadata_cid` (e.g. `ipfs://`). Clients join
    /// the two off-chain; Soroban strings cannot be concatenated on-chain.
    pub base_uri: String,
}

/// Mint arguments describing the badge itself, kept in one struct so
/// `mint_badge` stays a four argument call.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BadgeSpec {
    pub kind: BadgeKind,
    pub reference: u64,
    /// IPFS CID of the badge artwork / attributes JSON.
    pub metadata_cid: String,
    /// `true` makes the badge non-transferable for its whole life.
    pub soulbound: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Badge {
    pub id: u64,
    /// Current holder. Equal to `minted_to` until a transferable badge moves.
    pub owner: Address,
    /// Original recipient, kept so the one-per-milestone guard survives transfers.
    pub minted_to: Address,
    pub kind: BadgeKind,
    pub reference: u64,
    pub metadata_cid: String,
    pub soulbound: bool,
    pub minted_at: u64,
    pub minted_by: Address,
}

#[contracttype]
pub enum DataKey {
    /// Admin address; manages the minter allow-list and may always mint.
    Admin,
    /// Collection name / symbol / base URI.
    Collection,
    /// Minter allow-list entry (reputation admin, quid-store payout hook, ...).
    Minter(Address),
    /// Badge record by id.
    Badge(u64),
    /// Monotonic badge id counter; never decremented on burn.
    BadgeCount,
    /// Badge ids held by an address, for `list_by_owner`.
    Owned(Address),
    /// Uniqueness guard so a retried payout hook cannot double mint:
    /// (original recipient, kind, reference) -> badge id.
    Claimed(Address, BadgeKind, u64),
}
