#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, token, Address, Env, String};

mod error;
mod types;

use error::QuestBundleError;
use types::{Bundle, BundleStatus, DataKey};

const RECORD_TTL_LEDGERS: u32 = 5_184_000;

#[contractevent(topics = ["bundle", "created"])]
pub struct BundleCreatedEvent {
    pub bundle_id: u64,
    pub owner: Address,
    pub bonus_token: Address,
    pub escrow_amount: i128,
}

#[contractevent(topics = ["bundle", "mission"])]
pub struct MissionAddedEvent {
    pub bundle_id: u64,
    pub mission_id: u64,
    pub mission_count: u32,
}

#[contractevent(topics = ["bundle", "completion"])]
pub struct CompletionRecordedEvent {
    pub bundle_id: u64,
    pub hunter: Address,
    pub mission_id: u64,
    pub completed_count: u32,
}

#[contractevent(topics = ["bundle", "claimed"])]
pub struct BonusClaimedEvent {
    pub bundle_id: u64,
    pub hunter: Address,
    pub token: Address,
    pub amount: i128,
}

#[contractevent(topics = ["bundle", "cancelled"])]
pub struct BundleCancelledEvent {
    pub bundle_id: u64,
    pub owner: Address,
    pub refund_amount: i128,
}

/// Multi-mission campaigns over `quid-store` missions.
///
/// A founder bundles the mission ids of a sequence of feedback tasks under
/// shared campaign metadata, optionally escrowing a bonus paid to each hunter
/// who finishes the whole set. Three rules carry the design:
///
/// 1. **A bonus is only ever paid out of escrow.** `create_bundle` pulls
///    `bonus_amount * max_claims` from the founder up front and every claim
///    draws that balance down, so the contract can never owe more than it
///    holds and `cancel_bundle` can always refund the exact remainder.
///
/// 2. **Completions are reported, not asserted.** Only the configured payout
///    hook - the deployed `quid-store` contract - may call `record_completion`,
///    and only once per (bundle, hunter, mission). This mirrors how
///    `quid-referral` and `quid-badge-nft` learn about settled payouts: a
///    hunter cannot mark their own work done.
///
/// 3. **The required set is frozen once the campaign pays out.** Missions can
///    be added while a bundle is empty-handed, but the first claim fixes
///    `mission_count`, so nobody's finished campaign can be moved out of reach.
#[contract]
pub struct QuidQuestBundleContract;

#[contractimpl]
impl QuidQuestBundleContract {
    // -------------------------------------------------------------------------
    // Admin bootstrap
    // -------------------------------------------------------------------------

    pub fn initialize(env: Env, admin: Address) -> Result<(), QuestBundleError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(QuestBundleError::AlreadyInitialized);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, QuestBundleError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(QuestBundleError::NotInitialized)
    }

    pub fn set_admin(
        env: Env,
        caller: Address,
        new_admin: Address,
    ) -> Result<(), QuestBundleError> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        Ok(())
    }

    /// Set the only address allowed to report mission completions - in
    /// production the deployed `quid-store` contract. Admin only.
    pub fn set_payout_hook(
        env: Env,
        caller: Address,
        payout_hook: Address,
    ) -> Result<(), QuestBundleError> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::PayoutHook, &payout_hook);
        Ok(())
    }

    pub fn get_payout_hook(env: Env) -> Result<Address, QuestBundleError> {
        env.storage()
            .instance()
            .get(&DataKey::PayoutHook)
            .ok_or(QuestBundleError::NotInitialized)
    }

    // -------------------------------------------------------------------------
    // Campaign lifecycle
    // -------------------------------------------------------------------------

    /// Open a campaign and escrow its completion bonus.
    ///
    /// `bonus_amount * max_claims` moves from the founder into the contract now,
    /// so the bonus is funded for the whole campaign before a single hunter
    /// starts. Pass `bonus_amount = 0` for a campaign with no bonus - the
    /// completion is then tracked and published without any escrow.
    pub fn create_bundle(
        env: Env,
        owner: Address,
        title: String,
        metadata_cid: String,
        bonus_token: Address,
        bonus_amount: i128,
        max_claims: u32,
    ) -> Result<u64, QuestBundleError> {
        owner.require_auth();

        if bonus_amount < 0 {
            return Err(QuestBundleError::InvalidAmount);
        }
        // A funded bonus with nowhere to go is a mistake, not an empty campaign.
        if bonus_amount > 0 && max_claims == 0 {
            return Err(QuestBundleError::InvalidInput);
        }

        let escrow_amount = bonus_amount
            .checked_mul(max_claims as i128)
            .ok_or(QuestBundleError::Overflow)?;

        if escrow_amount > 0 {
            token::Client::new(&env, &bonus_token).transfer(
                &owner,
                env.current_contract_address(),
                &escrow_amount,
            );
        }

        let bundle_id = Self::next_bundle_id(&env);
        let bundle = Bundle {
            id: bundle_id,
            owner: owner.clone(),
            title,
            metadata_cid,
            bonus_token: bonus_token.clone(),
            bonus_amount,
            max_claims,
            claims_made: 0,
            escrow_balance: escrow_amount,
            mission_count: 0,
            created_at: env.ledger().timestamp(),
            status: BundleStatus::Active,
        };
        Self::save_bundle(&env, &bundle);

        BundleCreatedEvent {
            bundle_id,
            owner,
            bonus_token,
            escrow_amount,
        }
        .publish(&env);

        Ok(bundle_id)
    }

    /// Add a `quid-store` mission id to the campaign's required set.
    ///
    /// Rejected once a hunter has claimed, so the bar a finished campaign was
    /// measured against cannot be raised afterwards.
    pub fn add_mission(env: Env, bundle_id: u64, mission_id: u64) -> Result<u32, QuestBundleError> {
        let mut bundle = Self::get_bundle(env.clone(), bundle_id)?;
        bundle.owner.require_auth();
        Self::require_active(&bundle)?;

        if bundle.claims_made > 0 {
            return Err(QuestBundleError::InvalidState);
        }

        let membership_key = DataKey::BundleMission(bundle_id, mission_id);
        if env.storage().persistent().has(&membership_key) {
            return Err(QuestBundleError::MissionAlreadyAdded);
        }

        let index = bundle.mission_count;
        env.storage().persistent().set(&membership_key, &index);
        Self::extend(&env, &membership_key);

        let order_key = DataKey::BundleMissionAt(bundle_id, index);
        env.storage().persistent().set(&order_key, &mission_id);
        Self::extend(&env, &order_key);

        bundle.mission_count = index.checked_add(1).ok_or(QuestBundleError::Overflow)?;
        let mission_count = bundle.mission_count;
        Self::save_bundle(&env, &bundle);

        MissionAddedEvent {
            bundle_id,
            mission_id,
            mission_count,
        }
        .publish(&env);

        Ok(mission_count)
    }

    /// Report that `hunter` finished `mission_id` of this bundle.
    ///
    /// Callable only by the configured payout hook, and only once per
    /// (bundle, hunter, mission) - the dedupe slot is what keeps
    /// `CompletedCount` an honest count of distinct missions.
    pub fn record_completion(
        env: Env,
        caller: Address,
        bundle_id: u64,
        hunter: Address,
        mission_id: u64,
    ) -> Result<u32, QuestBundleError> {
        caller.require_auth();

        if caller != Self::get_payout_hook(env.clone())? {
            return Err(QuestBundleError::NotAuthorized);
        }

        let bundle = Self::get_bundle(env.clone(), bundle_id)?;
        Self::require_active(&bundle)?;

        if !env
            .storage()
            .persistent()
            .has(&DataKey::BundleMission(bundle_id, mission_id))
        {
            return Err(QuestBundleError::MissionNotInBundle);
        }

        let completion_key = DataKey::Completion(bundle_id, hunter.clone(), mission_id);
        if env.storage().persistent().has(&completion_key) {
            return Err(QuestBundleError::CompletionAlreadyRecorded);
        }
        env.storage().persistent().set(&completion_key, &true);
        Self::extend(&env, &completion_key);

        let count_key = DataKey::CompletedCount(bundle_id, hunter.clone());
        let completed_count = Self::get_completed_count(env.clone(), bundle_id, hunter.clone())
            .checked_add(1)
            .ok_or(QuestBundleError::Overflow)?;
        env.storage().persistent().set(&count_key, &completed_count);
        Self::extend(&env, &count_key);

        CompletionRecordedEvent {
            bundle_id,
            hunter,
            mission_id,
            completed_count,
        }
        .publish(&env);

        Ok(completed_count)
    }

    /// Pay `hunter` the completion bonus for finishing every mission.
    ///
    /// Returns the amount transferred, which is zero for a campaign that was
    /// created without a bonus. The claim is still recorded in that case, so a
    /// completion is a one-shot event either way.
    pub fn claim_completion_bonus(
        env: Env,
        bundle_id: u64,
        hunter: Address,
    ) -> Result<i128, QuestBundleError> {
        hunter.require_auth();

        let mut bundle = Self::get_bundle(env.clone(), bundle_id)?;
        Self::require_active(&bundle)?;

        if bundle.mission_count == 0 {
            return Err(QuestBundleError::BundleEmpty);
        }

        let claimed_key = DataKey::Claimed(bundle_id, hunter.clone());
        if env.storage().persistent().has(&claimed_key) {
            return Err(QuestBundleError::AlreadyClaimed);
        }

        if Self::get_completed_count(env.clone(), bundle_id, hunter.clone()) < bundle.mission_count
        {
            return Err(QuestBundleError::MissionsIncomplete);
        }

        let amount = bundle.bonus_amount;
        if amount > 0 {
            if bundle.claims_made >= bundle.max_claims {
                return Err(QuestBundleError::NoClaimsRemaining);
            }
            if bundle.escrow_balance < amount {
                return Err(QuestBundleError::InsufficientEscrow);
            }
        }

        // Book the claim before transferring: a repeated or re-entrant call
        // finds the slot taken and the escrow already drawn down.
        env.storage().persistent().set(&claimed_key, &true);
        Self::extend(&env, &claimed_key);

        bundle.claims_made = bundle
            .claims_made
            .checked_add(1)
            .ok_or(QuestBundleError::Overflow)?;
        bundle.escrow_balance = bundle
            .escrow_balance
            .checked_sub(amount)
            .ok_or(QuestBundleError::Overflow)?;
        Self::save_bundle(&env, &bundle);

        if amount > 0 {
            token::Client::new(&env, &bundle.bonus_token).transfer(
                &env.current_contract_address(),
                &hunter,
                &amount,
            );
        }

        BonusClaimedEvent {
            bundle_id,
            hunter,
            token: bundle.bonus_token,
            amount,
        }
        .publish(&env);

        Ok(amount)
    }

    /// Close the campaign and refund whatever bonus is still escrowed.
    ///
    /// Bonuses already claimed are untouched; the founder gets back exactly the
    /// unclaimed remainder, and no further claim or completion can land.
    pub fn cancel_bundle(env: Env, bundle_id: u64) -> Result<i128, QuestBundleError> {
        let mut bundle = Self::get_bundle(env.clone(), bundle_id)?;
        bundle.owner.require_auth();
        Self::require_active(&bundle)?;

        let refund_amount = bundle.escrow_balance;

        // Zero the escrow before transferring, so a refund can never be taken
        // twice even if the token contract calls back in.
        bundle.escrow_balance = 0;
        bundle.status = BundleStatus::Cancelled;
        Self::save_bundle(&env, &bundle);

        if refund_amount > 0 {
            token::Client::new(&env, &bundle.bonus_token).transfer(
                &env.current_contract_address(),
                &bundle.owner,
                &refund_amount,
            );
        }

        BundleCancelledEvent {
            bundle_id,
            owner: bundle.owner,
            refund_amount,
        }
        .publish(&env);

        Ok(refund_amount)
    }

    // -------------------------------------------------------------------------
    // Views
    // -------------------------------------------------------------------------

    pub fn get_bundle(env: Env, bundle_id: u64) -> Result<Bundle, QuestBundleError> {
        env.storage()
            .persistent()
            .get(&DataKey::Bundle(bundle_id))
            .ok_or(QuestBundleError::BundleNotFound)
    }

    pub fn get_bundle_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::BundleCount)
            .unwrap_or(0)
    }

    /// The mission id at `index` in the campaign's ordered list.
    pub fn get_mission_at(env: Env, bundle_id: u64, index: u32) -> Result<u64, QuestBundleError> {
        env.storage()
            .persistent()
            .get(&DataKey::BundleMissionAt(bundle_id, index))
            .ok_or(QuestBundleError::MissionNotInBundle)
    }

    pub fn is_mission_in_bundle(env: Env, bundle_id: u64, mission_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::BundleMission(bundle_id, mission_id))
    }

    pub fn get_completed_count(env: Env, bundle_id: u64, hunter: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::CompletedCount(bundle_id, hunter))
            .unwrap_or(0)
    }

    pub fn is_completed(env: Env, bundle_id: u64, hunter: Address, mission_id: u64) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Completion(bundle_id, hunter, mission_id))
    }

    pub fn has_claimed(env: Env, bundle_id: u64, hunter: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::Claimed(bundle_id, hunter))
    }

    /// Whether `hunter` has finished every mission the bundle requires.
    ///
    /// False for an empty bundle: a campaign with no missions is not something
    /// anyone can have completed.
    pub fn is_bundle_complete_for(env: Env, bundle_id: u64, hunter: Address) -> bool {
        let Ok(bundle) = Self::get_bundle(env.clone(), bundle_id) else {
            return false;
        };
        bundle.mission_count > 0
            && Self::get_completed_count(env, bundle_id, hunter) >= bundle.mission_count
    }

    // -------------------------------------------------------------------------
    // Internals
    // -------------------------------------------------------------------------

    fn require_admin(env: &Env, caller: &Address) -> Result<(), QuestBundleError> {
        caller.require_auth();
        if *caller != Self::get_admin(env.clone())? {
            return Err(QuestBundleError::NotAuthorized);
        }
        Ok(())
    }

    fn require_active(bundle: &Bundle) -> Result<(), QuestBundleError> {
        if bundle.status != BundleStatus::Active {
            return Err(QuestBundleError::InvalidState);
        }
        Ok(())
    }

    fn next_bundle_id(env: &Env) -> u64 {
        let count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::BundleCount)
            .unwrap_or(0)
            + 1;
        env.storage().instance().set(&DataKey::BundleCount, &count);
        count
    }

    fn save_bundle(env: &Env, bundle: &Bundle) {
        let key = DataKey::Bundle(bundle.id);
        env.storage().persistent().set(&key, bundle);
        Self::extend(env, &key);
    }

    fn extend(env: &Env, key: &DataKey) {
        env.storage()
            .persistent()
            .extend_ttl(key, RECORD_TTL_LEDGERS, RECORD_TTL_LEDGERS);
    }
}

mod test;
