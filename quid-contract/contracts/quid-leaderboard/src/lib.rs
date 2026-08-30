#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, Address, Env, Vec};

mod error;
mod types;

use error::LeaderboardError;
use types::{DataKey, Epoch, LeaderboardEntry};

/// How long (in ledgers) persistent entries live after their last access.
/// ~1 year at 5-second ledger close time.
const ENTRY_TTL_LEDGERS: u32 = 6_307_200;

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[contractevent(topics = ["epoch", "started"])]
pub struct EpochStartedEvent {
    pub epoch_id: u64,
    pub started_at: u64,
}

#[contractevent(topics = ["epoch", "ended"])]
pub struct EpochEndedEvent {
    pub epoch_id: u64,
    pub ended_at: u64,
    pub entry_count: u32,
}

#[contractevent(topics = ["score", "recorded"])]
pub struct ScoreRecordedEvent {
    pub epoch_id: u64,
    pub hunter: Address,
    pub new_score: i64,
}

// ---------------------------------------------------------------------------
// Contract
// ---------------------------------------------------------------------------

#[contract]
pub struct QuidLeaderboardContract;

#[contractimpl]
impl QuidLeaderboardContract {
    // -----------------------------------------------------------------------
    // Bootstrap
    // -----------------------------------------------------------------------

    /// Initialize the contract.  Must be called exactly once.
    /// `admin`    – can change the recorder and call admin-only functions.
    /// `recorder` – the only address allowed to call `record_score`.
    pub fn initialize(env: Env, admin: Address, recorder: Address) -> Result<(), LeaderboardError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(LeaderboardError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Recorder, &recorder);
        env.storage().instance().set(&DataKey::EpochCount, &0_u64);

        Ok(())
    }

    // -----------------------------------------------------------------------
    // View helpers
    // -----------------------------------------------------------------------

    pub fn get_admin(env: Env) -> Result<Address, LeaderboardError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(LeaderboardError::NotInitialized)
    }

    pub fn get_recorder(env: Env) -> Result<Address, LeaderboardError> {
        env.storage()
            .instance()
            .get(&DataKey::Recorder)
            .ok_or(LeaderboardError::NotInitialized)
    }

    pub fn get_active_epoch_id(env: Env) -> Result<u64, LeaderboardError> {
        env.storage()
            .instance()
            .get(&DataKey::ActiveEpochId)
            .ok_or(LeaderboardError::NoActiveEpoch)
    }

    pub fn get_epoch(env: Env, epoch_id: u64) -> Result<Epoch, LeaderboardError> {
        env.storage()
            .persistent()
            .get(&DataKey::Epoch(epoch_id))
            .ok_or(LeaderboardError::EpochNotFound)
    }

    pub fn get_score(env: Env, epoch_id: u64, hunter: Address) -> i64 {
        env.storage()
            .persistent()
            .get(&DataKey::Score(epoch_id, hunter))
            .unwrap_or(0_i64)
    }

    pub fn get_epoch_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::EpochCount)
            .unwrap_or(0)
    }

    // -----------------------------------------------------------------------
    // Admin: update recorder
    // -----------------------------------------------------------------------

    pub fn set_recorder(
        env: Env,
        caller: Address,
        new_recorder: Address,
    ) -> Result<(), LeaderboardError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::Recorder, &new_recorder);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Epoch lifecycle
    // -----------------------------------------------------------------------

    /// Start a new season epoch.  Only admin may call this.
    /// Panics if an epoch is already active.
    pub fn start_epoch(env: Env, caller: Address) -> Result<u64, LeaderboardError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        // Ensure no epoch is currently open.
        if env.storage().instance().has(&DataKey::ActiveEpochId) {
            return Err(LeaderboardError::EpochAlreadyActive);
        }

        let epoch_id = Self::next_epoch_id(&env);
        let started_at = env.ledger().timestamp();

        let epoch = Epoch {
            id: epoch_id,
            started_at,
            ended_at: 0,
            entry_count: 0,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Epoch(epoch_id), &epoch);
        env.storage().persistent().extend_ttl(
            &DataKey::Epoch(epoch_id),
            ENTRY_TTL_LEDGERS,
            ENTRY_TTL_LEDGERS,
        );

        // Store an empty hunters list for this epoch.
        let empty: Vec<Address> = Vec::new(&env);
        env.storage()
            .persistent()
            .set(&DataKey::Hunters(epoch_id), &empty);
        env.storage().persistent().extend_ttl(
            &DataKey::Hunters(epoch_id),
            ENTRY_TTL_LEDGERS,
            ENTRY_TTL_LEDGERS,
        );

        env.storage()
            .instance()
            .set(&DataKey::ActiveEpochId, &epoch_id);

        EpochStartedEvent {
            epoch_id,
            started_at,
        }
        .publish(&env);

        Ok(epoch_id)
    }

    /// Record (or add to) a hunter's score for the active epoch.
    /// Only the authorized recorder may call this.
    /// `delta` may be negative (penalty / correction).
    pub fn record_score(
        env: Env,
        caller: Address,
        hunter: Address,
        delta: i64,
    ) -> Result<i64, LeaderboardError> {
        caller.require_auth();
        Self::require_recorder(&env, &caller)?;

        let epoch_id = env
            .storage()
            .instance()
            .get::<DataKey, u64>(&DataKey::ActiveEpochId)
            .ok_or(LeaderboardError::NoActiveEpoch)?;

        // Update the hunter's score.
        let prev: i64 = env
            .storage()
            .persistent()
            .get(&DataKey::Score(epoch_id, hunter.clone()))
            .unwrap_or(0_i64);

        let new_score = prev
            .checked_add(delta)
            .ok_or(LeaderboardError::ScoreOverflow)?;

        env.storage()
            .persistent()
            .set(&DataKey::Score(epoch_id, hunter.clone()), &new_score);
        env.storage().persistent().extend_ttl(
            &DataKey::Score(epoch_id, hunter.clone()),
            ENTRY_TTL_LEDGERS,
            ENTRY_TTL_LEDGERS,
        );

        // Track the hunter in the epoch's roster if this is their first score.
        let mut hunters: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Hunters(epoch_id))
            .unwrap_or(Vec::new(&env));

        let already_tracked = hunters.iter().any(|h| h == hunter);
        if !already_tracked {
            hunters.push_back(hunter.clone());

            // Update entry_count in the epoch metadata.
            let mut epoch: Epoch = env
                .storage()
                .persistent()
                .get(&DataKey::Epoch(epoch_id))
                .ok_or(LeaderboardError::EpochNotFound)?;
            epoch.entry_count += 1;

            env.storage()
                .persistent()
                .set(&DataKey::Epoch(epoch_id), &epoch);
            env.storage().persistent().extend_ttl(
                &DataKey::Epoch(epoch_id),
                ENTRY_TTL_LEDGERS,
                ENTRY_TTL_LEDGERS,
            );
        }

        env.storage()
            .persistent()
            .set(&DataKey::Hunters(epoch_id), &hunters);
        env.storage().persistent().extend_ttl(
            &DataKey::Hunters(epoch_id),
            ENTRY_TTL_LEDGERS,
            ENTRY_TTL_LEDGERS,
        );

        ScoreRecordedEvent {
            epoch_id,
            hunter,
            new_score,
        }
        .publish(&env);

        Ok(new_score)
    }

    /// Return the top `n` hunters for `epoch_id`, sorted descending by score.
    /// Works for both active and completed epochs.
    pub fn get_top_n(
        env: Env,
        epoch_id: u64,
        n: u32,
    ) -> Result<Vec<LeaderboardEntry>, LeaderboardError> {
        // Validate the epoch exists.
        let epoch: Epoch = env
            .storage()
            .persistent()
            .get(&DataKey::Epoch(epoch_id))
            .ok_or(LeaderboardError::EpochNotFound)?;

        if n == 0 || n > epoch.entry_count {
            return Err(LeaderboardError::InvalidN);
        }

        let hunters: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::Hunters(epoch_id))
            .unwrap_or(Vec::new(&env));

        // Collect (score, hunter) pairs.
        let count = hunters.len();
        let mut pairs: Vec<(i64, Address)> = Vec::new(&env);
        for i in 0..count {
            let h = hunters.get(i).unwrap();
            let s: i64 = env
                .storage()
                .persistent()
                .get(&DataKey::Score(epoch_id, h.clone()))
                .unwrap_or(0_i64);
            pairs.push_back((s, h));
        }

        // Insertion sort (descending by score).  On-chain n is expected to be
        // small (≤ 100 hunters per epoch in practice), so O(n²) is acceptable.
        let len = pairs.len();
        for i in 1..len {
            let mut j = i;
            while j > 0 {
                let a = pairs.get(j - 1).unwrap();
                let b = pairs.get(j).unwrap();
                if a.0 >= b.0 {
                    break;
                }
                pairs.set(j - 1, b);
                pairs.set(j, a);
                j -= 1;
            }
        }

        // Build the result slice of length n.
        let mut result: Vec<LeaderboardEntry> = Vec::new(&env);
        for i in 0..n {
            let (score, hunter) = pairs.get(i).unwrap();
            result.push_back(LeaderboardEntry {
                rank: i + 1,
                hunter,
                score,
            });
        }

        Ok(result)
    }

    /// Close the active epoch.  Only admin may call this.
    pub fn end_epoch(env: Env, caller: Address) -> Result<u64, LeaderboardError> {
        caller.require_auth();
        Self::require_admin(&env, &caller)?;

        let epoch_id: u64 = env
            .storage()
            .instance()
            .get(&DataKey::ActiveEpochId)
            .ok_or(LeaderboardError::NoActiveEpoch)?;

        let ended_at = env.ledger().timestamp();

        let mut epoch: Epoch = env
            .storage()
            .persistent()
            .get(&DataKey::Epoch(epoch_id))
            .ok_or(LeaderboardError::EpochNotFound)?;

        epoch.ended_at = ended_at;

        env.storage()
            .persistent()
            .set(&DataKey::Epoch(epoch_id), &epoch);
        env.storage().persistent().extend_ttl(
            &DataKey::Epoch(epoch_id),
            ENTRY_TTL_LEDGERS,
            ENTRY_TTL_LEDGERS,
        );

        // Clear the active epoch pointer.
        env.storage().instance().remove(&DataKey::ActiveEpochId);

        EpochEndedEvent {
            epoch_id,
            ended_at,
            entry_count: epoch.entry_count,
        }
        .publish(&env);

        Ok(epoch_id)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn require_admin(env: &Env, caller: &Address) -> Result<(), LeaderboardError> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(LeaderboardError::NotInitialized)?;

        if *caller != admin {
            return Err(LeaderboardError::NotAuthorized);
        }

        Ok(())
    }

    fn require_recorder(env: &Env, caller: &Address) -> Result<(), LeaderboardError> {
        let recorder: Address = env
            .storage()
            .instance()
            .get(&DataKey::Recorder)
            .ok_or(LeaderboardError::NotInitialized)?;

        if *caller != recorder {
            return Err(LeaderboardError::NotAuthorized);
        }

        Ok(())
    }

    fn next_epoch_id(env: &Env) -> u64 {
        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::EpochCount)
            .unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::EpochCount, &count);
        count
    }
}

mod test;
