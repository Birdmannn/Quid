use soroban_sdk::{contracttype, Address, BytesN, String};

/// A hunter's one and only referral record.
///
/// The link itself lives off-chain; `link_cid` points at it and `link_hash` is
/// the on-chain commitment to that CID, so an indexer can match a record to its
/// blob without trusting the string it was handed.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Referral {
    pub referrer: Address,
    pub referred: Address,
    pub link_cid: String,
    pub link_hash: BytesN<32>,
    pub registered_at: u64,
}

#[contracttype]
pub enum DataKey {
    Admin,
    /// Referral reward rate in basis points (1 bps = 0.01%).
    RewardBps,
    /// The only address allowed to report successful payouts.
    PayoutHook,
    /// Referral record, keyed by the referred hunter so a hunter has exactly
    /// one referrer for life.
    Referral(Address),
    /// How many hunters an address has referred.
    ReferredCount(Address),
    /// Unclaimed reward for a referrer in one token.
    Claimable(Address, Address),
    /// Lifetime claimed reward for a referrer in one token.
    Claimed(Address, Address),
    /// Marks a (mission, hunter) payout as already accrued.
    PayoutRecorded(u64, Address),
}
