#![no_std]
use soroban_sdk::{
    contract, contractevent, contractimpl, token, xdr::ToXdr, Address, BytesN, Env, String,
};

mod error;
mod types;

use error::ReferralError;
use types::{DataKey, Referral};

/// 100% expressed in basis points.
pub const MAX_REWARD_BPS: u32 = 10_000;

const RECORD_TTL_LEDGERS: u32 = 5_184_000;

#[contractevent(topics = ["referral", "registered"])]
pub struct ReferralRegisteredEvent {
    pub referrer: Address,
    pub referred: Address,
    pub link_hash: BytesN<32>,
}

#[contractevent(topics = ["referral", "accrued"])]
pub struct RewardAccruedEvent {
    pub referrer: Address,
    pub referred: Address,
    pub mission_id: u64,
    pub token: Address,
    pub amount: i128,
}

#[contractevent(topics = ["referral", "claimed"])]
pub struct RewardClaimedEvent {
    pub referrer: Address,
    pub token: Address,
    pub amount: i128,
}

#[contractevent(topics = ["referral", "bps"], data_format = "single-value")]
pub struct RewardBpsUpdatedEvent {
    pub reward_bps: u32,
}

/// Referral rewards for founder/hunter acquisition.
///
/// A hunter registers who referred them; the referrer only ever accrues a
/// reward once that hunter is actually paid for a mission, which is reported by
/// the payout hook (`quid-store`). Rewards are paid from a pool anyone can top
/// up with `fund`, and are never minted out of thin air.
#[contract]
pub struct QuidReferralContract;

#[contractimpl]
impl QuidReferralContract {
    // -------------------------------------------------------------------------
    // Admin bootstrap
    // -------------------------------------------------------------------------

    pub fn initialize(env: Env, admin: Address, reward_bps: u32) -> Result<(), ReferralError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(ReferralError::AlreadyInitialized);
        }
        if reward_bps > MAX_REWARD_BPS {
            return Err(ReferralError::InvalidRewardBps);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::RewardBps, &reward_bps);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, ReferralError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(ReferralError::NotInitialized)
    }

    pub fn set_admin(env: Env, caller: Address, new_admin: Address) -> Result<(), ReferralError> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Reward rate
    // -------------------------------------------------------------------------

    /// Set the referral reward rate in basis points. Admin only.
    ///
    /// Only affects rewards accrued after the change; already-accrued balances
    /// stay claimable at the rate they were earned under.
    pub fn set_reward_bps(env: Env, caller: Address, reward_bps: u32) -> Result<(), ReferralError> {
        Self::require_admin(&env, &caller)?;

        if reward_bps > MAX_REWARD_BPS {
            return Err(ReferralError::InvalidRewardBps);
        }

        env.storage()
            .instance()
            .set(&DataKey::RewardBps, &reward_bps);

        RewardBpsUpdatedEvent { reward_bps }.publish(&env);

        Ok(())
    }

    pub fn get_reward_bps(env: Env) -> u32 {
        env.storage()
            .instance()
            .get(&DataKey::RewardBps)
            .unwrap_or(0)
    }

    /// Reward owed on a payout of `payout_amount`, rounded down.
    pub fn compute_reward(env: Env, payout_amount: i128) -> Result<i128, ReferralError> {
        if payout_amount < 0 {
            return Err(ReferralError::InvalidAmount);
        }

        let reward_bps = Self::get_reward_bps(env) as i128;

        payout_amount
            .checked_mul(reward_bps)
            .map(|scaled| scaled / MAX_REWARD_BPS as i128)
            .ok_or(ReferralError::Overflow)
    }

    // -------------------------------------------------------------------------
    // Payout hook
    // -------------------------------------------------------------------------

    /// Set the only address allowed to report payouts — in production the
    /// deployed `quid-store` contract. Admin only.
    pub fn set_payout_hook(
        env: Env,
        caller: Address,
        payout_hook: Address,
    ) -> Result<(), ReferralError> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .instance()
            .set(&DataKey::PayoutHook, &payout_hook);
        Ok(())
    }

    pub fn get_payout_hook(env: Env) -> Result<Address, ReferralError> {
        env.storage()
            .instance()
            .get(&DataKey::PayoutHook)
            .ok_or(ReferralError::NotAuthorized)
    }

    // -------------------------------------------------------------------------
    // Registration
    // -------------------------------------------------------------------------

    /// Record that `referred` arrived through `referrer`'s link.
    ///
    /// The referred hunter authorizes, so nobody can be claimed as a referral
    /// against their will. A hunter can only ever be registered once.
    pub fn register_referral(
        env: Env,
        referrer: Address,
        referred: Address,
        link_cid: String,
    ) -> Result<(), ReferralError> {
        referred.require_auth();

        if referrer == referred {
            return Err(ReferralError::SelfReferral);
        }
        if link_cid.is_empty() {
            return Err(ReferralError::InvalidInput);
        }

        let key = DataKey::Referral(referred.clone());
        if env.storage().persistent().has(&key) {
            return Err(ReferralError::AlreadyRegistered);
        }

        let link_hash = env
            .crypto()
            .sha256(&link_cid.clone().to_xdr(&env))
            .to_bytes();

        let referral = Referral {
            referrer: referrer.clone(),
            referred: referred.clone(),
            link_cid,
            link_hash: link_hash.clone(),
            registered_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&key, &referral);
        env.storage()
            .persistent()
            .extend_ttl(&key, RECORD_TTL_LEDGERS, RECORD_TTL_LEDGERS);

        let count_key = DataKey::ReferredCount(referrer.clone());
        let count: u32 = env.storage().persistent().get(&count_key).unwrap_or(0);
        env.storage().persistent().set(&count_key, &(count + 1));
        env.storage()
            .persistent()
            .extend_ttl(&count_key, RECORD_TTL_LEDGERS, RECORD_TTL_LEDGERS);

        ReferralRegisteredEvent {
            referrer,
            referred,
            link_hash,
        }
        .publish(&env);

        Ok(())
    }

    pub fn get_referral(env: Env, referred: Address) -> Result<Referral, ReferralError> {
        env.storage()
            .persistent()
            .get(&DataKey::Referral(referred))
            .ok_or(ReferralError::ReferralNotFound)
    }

    pub fn has_referral(env: Env, referred: Address) -> bool {
        env.storage().persistent().has(&DataKey::Referral(referred))
    }

    pub fn get_referred_count(env: Env, referrer: Address) -> u32 {
        env.storage()
            .persistent()
            .get(&DataKey::ReferredCount(referrer))
            .unwrap_or(0)
    }

    // -------------------------------------------------------------------------
    // Accrual (payout hook)
    // -------------------------------------------------------------------------

    /// Report that `referred` was successfully paid `payout_amount` for
    /// `mission_id`, accruing their referrer's cut.
    ///
    /// This is the only path that ever grows a claimable balance, which is what
    /// makes "reward paid only after successful payout" hold. Callable only by
    /// the configured payout hook, and only once per (mission, hunter).
    ///
    /// Returns the reward accrued — zero when the hunter was never referred, so
    /// the store can call this for every payout without special-casing.
    pub fn record_payout(
        env: Env,
        caller: Address,
        referred: Address,
        mission_id: u64,
        token: Address,
        payout_amount: i128,
    ) -> Result<i128, ReferralError> {
        caller.require_auth();

        if caller != Self::get_payout_hook(env.clone())? {
            return Err(ReferralError::NotAuthorized);
        }
        if payout_amount <= 0 {
            return Err(ReferralError::InvalidAmount);
        }

        let recorded_key = DataKey::PayoutRecorded(mission_id, referred.clone());
        if env.storage().persistent().has(&recorded_key) {
            return Err(ReferralError::PayoutAlreadyRecorded);
        }

        // Claim the (mission, hunter) slot even when there is no referrer, so a
        // referral registered after the fact cannot be back-paid for it.
        env.storage().persistent().set(&recorded_key, &true);
        env.storage().persistent().extend_ttl(
            &recorded_key,
            RECORD_TTL_LEDGERS,
            RECORD_TTL_LEDGERS,
        );

        let Some(referral) = env
            .storage()
            .persistent()
            .get::<_, Referral>(&DataKey::Referral(referred.clone()))
        else {
            return Ok(0);
        };

        let reward = Self::compute_reward(env.clone(), payout_amount)?;
        if reward == 0 {
            return Ok(0);
        }

        let key = DataKey::Claimable(referral.referrer.clone(), token.clone());
        let claimable = Self::get_claimable(env.clone(), referral.referrer.clone(), token.clone())
            .checked_add(reward)
            .ok_or(ReferralError::Overflow)?;
        env.storage().persistent().set(&key, &claimable);
        env.storage()
            .persistent()
            .extend_ttl(&key, RECORD_TTL_LEDGERS, RECORD_TTL_LEDGERS);

        RewardAccruedEvent {
            referrer: referral.referrer,
            referred,
            mission_id,
            token,
            amount: reward,
        }
        .publish(&env);

        Ok(reward)
    }

    pub fn is_payout_recorded(env: Env, mission_id: u64, referred: Address) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::PayoutRecorded(mission_id, referred))
    }

    // -------------------------------------------------------------------------
    // Reward pool
    // -------------------------------------------------------------------------

    /// Top up the pool rewards are paid from.
    pub fn fund(
        env: Env,
        from: Address,
        token: Address,
        amount: i128,
    ) -> Result<(), ReferralError> {
        from.require_auth();

        if amount <= 0 {
            return Err(ReferralError::InvalidAmount);
        }

        token::Client::new(&env, &token).transfer(&from, env.current_contract_address(), &amount);

        Ok(())
    }

    pub fn get_pool_balance(env: Env, token: Address) -> i128 {
        token::Client::new(&env, &token).balance(&env.current_contract_address())
    }

    // -------------------------------------------------------------------------
    // Claiming
    // -------------------------------------------------------------------------

    /// Pay out everything `referrer` has accrued in `token`.
    ///
    /// The balance is zeroed before the transfer, so a re-entrant claim finds
    /// nothing left to take.
    pub fn claim_reward(
        env: Env,
        referrer: Address,
        token: Address,
    ) -> Result<i128, ReferralError> {
        referrer.require_auth();

        let amount = Self::get_claimable(env.clone(), referrer.clone(), token.clone());
        if amount <= 0 {
            return Err(ReferralError::NothingToClaim);
        }
        if Self::get_pool_balance(env.clone(), token.clone()) < amount {
            return Err(ReferralError::InsufficientFunds);
        }

        let claimable_key = DataKey::Claimable(referrer.clone(), token.clone());
        env.storage().persistent().set(&claimable_key, &0i128);

        let claimed_key = DataKey::Claimed(referrer.clone(), token.clone());
        let claimed = Self::get_claimed(env.clone(), referrer.clone(), token.clone())
            .checked_add(amount)
            .ok_or(ReferralError::Overflow)?;
        env.storage().persistent().set(&claimed_key, &claimed);
        env.storage()
            .persistent()
            .extend_ttl(&claimed_key, RECORD_TTL_LEDGERS, RECORD_TTL_LEDGERS);

        token::Client::new(&env, &token).transfer(
            &env.current_contract_address(),
            &referrer,
            &amount,
        );

        RewardClaimedEvent {
            referrer,
            token,
            amount,
        }
        .publish(&env);

        Ok(amount)
    }

    pub fn get_claimable(env: Env, referrer: Address, token: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Claimable(referrer, token))
            .unwrap_or(0)
    }

    pub fn get_claimed(env: Env, referrer: Address, token: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Claimed(referrer, token))
            .unwrap_or(0)
    }

    // -------------------------------------------------------------------------
    // Internals
    // -------------------------------------------------------------------------

    fn require_admin(env: &Env, caller: &Address) -> Result<(), ReferralError> {
        caller.require_auth();

        let admin = Self::get_admin(env.clone())?;
        if *caller != admin {
            return Err(ReferralError::NotAuthorized);
        }

        Ok(())
    }
}

mod test;
