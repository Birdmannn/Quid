use soroban_sdk::{contracttype, Address};

// ---------------------------------------------------------------------------
// Storage keys
// ---------------------------------------------------------------------------

#[contracttype]
pub enum DataKey {
    /// The admin address (instance storage).
    Admin,
    /// The authorized score recorder address (instance storage).
    Recorder,
    /// The ID of the currently active epoch, if any (instance storage).
    ActiveEpochId,
    /// Counter used to mint monotonically increasing epoch IDs (instance storage).
    EpochCount,
    /// Metadata for a specific epoch (persistent storage).
    Epoch(u64),
    /// Score for (epoch_id, hunter) (persistent storage).
    Score(u64, Address),
    /// Ordered list of unique hunters who have a score in an epoch (persistent storage).
    /// Stored as Vec<Address> but we do not sort on-chain; sorting happens off-chain
    /// or we keep a sorted vec in record_score.
    Hunters(u64),
}

// ---------------------------------------------------------------------------
// Epoch metadata
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Epoch {
    pub id: u64,
    /// Ledger timestamp when start_epoch was called.
    pub started_at: u64,
    /// Ledger timestamp when end_epoch was called; 0 means still active.
    pub ended_at: u64,
    /// Number of distinct hunters with a score in this epoch.
    pub entry_count: u32,
}

// ---------------------------------------------------------------------------
// Public leaderboard entry returned by get_top_n
// ---------------------------------------------------------------------------

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub hunter: Address,
    pub score: i64,
}
