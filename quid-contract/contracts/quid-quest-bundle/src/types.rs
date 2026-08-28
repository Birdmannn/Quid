use soroban_sdk::{contracttype, Address, String};

#[derive(Clone, Debug, Default, PartialEq, Eq, Copy)]
#[contracttype]
pub enum BundleStatus {
    #[default]
    Active,
    Cancelled,
}

/// A multi-mission campaign.
///
/// The missions themselves stay in `quid-store`; a bundle only references their
/// ids and layers shared campaign metadata plus an optional completion bonus on
/// top. `escrow_balance` is the contract's own accounting of what it still owes
/// for this bundle, so a cancellation refunds exactly the unclaimed remainder.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Bundle {
    pub id: u64,
    pub owner: Address,
    pub title: String,
    /// Shared campaign metadata, off-chain (IPFS CID).
    pub metadata_cid: String,
    pub bonus_token: Address,
    /// Bonus per hunter who finishes every mission. Zero means no bonus.
    pub bonus_amount: i128,
    /// How many hunters can be paid the bonus.
    pub max_claims: u32,
    pub claims_made: u32,
    /// Escrowed bonus not yet paid out or refunded.
    pub escrow_balance: i128,
    /// Number of missions required for completion.
    pub mission_count: u32,
    pub created_at: u64,
    pub status: BundleStatus,
}

#[contracttype]
pub enum DataKey {
    Admin,
    /// The only address allowed to report mission completions - in production
    /// the deployed `quid-store` contract.
    PayoutHook,
    Bundle(u64),
    BundleCount,
    /// Membership marker: `quid-store` mission id -> its index in the bundle.
    BundleMission(u64, u64),
    /// Ordered mission ids, so a client can enumerate a campaign.
    BundleMissionAt(u64, u32),
    /// Marks one (bundle, hunter, mission) as completed.
    Completion(u64, Address, u64),
    /// How many of the bundle's missions this hunter has completed.
    CompletedCount(u64, Address),
    /// Marks a hunter as having taken the completion bonus.
    Claimed(u64, Address),
}
