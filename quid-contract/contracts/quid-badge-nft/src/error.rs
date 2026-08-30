use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BadgeError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    /// Caller is neither the admin nor on the minter allow-list.
    NotAuthorized = 3,
    BadgeNotFound = 4,
    /// This owner already holds a badge for the same (kind, reference) pair.
    AlreadyMinted = 5,
    /// Metadata CID is empty or longer than `MAX_CID_LEN`.
    InvalidMetadata = 6,
    /// Soulbound badges can never change owner.
    SoulboundBadge = 7,
    NotBadgeOwner = 8,
    SelfTransfer = 9,
    /// Owner already holds `MAX_BADGES_PER_OWNER` badges.
    OwnerBadgeLimit = 10,
    AlreadyMinter = 11,
    MinterNotFound = 12,
    /// Collection name or symbol is empty.
    InvalidCollection = 13,
}
