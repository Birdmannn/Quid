#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, token, Address, Env};

mod error;
mod types;

use error::DisputeError;
use types::{CreateDisputeParams, DataKey, Dispute, DisputeStatus};

const TTL_LEDGERS: u32 = 5_184_000;

#[contractevent(topics = ["dispute", "created"])]
pub struct DisputeCreatedEvent {
    pub dispute_id: u64,
    pub mission_id: u64,
    pub hunter: Address,
    pub respondent: Address,
}

#[contractevent(topics = ["dispute", "bond"])]
pub struct BondStakedEvent {
    pub dispute_id: u64,
    pub staker: Address,
    pub amount: i128,
}

#[contractevent(topics = ["dispute", "resolved"])]
pub struct DisputeResolvedEvent {
    pub dispute_id: u64,
    pub winner: Address,
    pub hunter_wins: bool,
}

#[contractevent(topics = ["dispute", "timeout"])]
pub struct DisputeTimeoutEvent {
    pub dispute_id: u64,
    pub hunter: Address,
}

#[contract]
pub struct QuidDisputeContract;

#[contractimpl]
impl QuidDisputeContract {
    /// Open a dispute against a contested submission and stake the hunter bond.
    /// Optional `arbiter` may later call `resolve_by_arbiter` before the deadline.
    pub fn create_dispute(
        env: Env,
        hunter: Address,
        params: CreateDisputeParams,
    ) -> Result<u64, DisputeError> {
        hunter.require_auth();

        if params.bond_amount <= 0 {
            return Err(DisputeError::InvalidAmount);
        }
        if params.timeout_secs == 0 {
            return Err(DisputeError::InvalidTimeout);
        }
        if hunter == params.respondent {
            return Err(DisputeError::InvalidParties);
        }
        if let Some(ref arbiter) = params.arbiter {
            if *arbiter == hunter || *arbiter == params.respondent {
                return Err(DisputeError::InvalidParties);
            }
        }

        let open_key = DataKey::OpenDispute(params.mission_id, hunter.clone());
        if env.storage().persistent().has(&open_key) {
            return Err(DisputeError::AlreadyDisputed);
        }

        token::Client::new(&env, &params.bond_token).transfer(
            &hunter,
            env.current_contract_address(),
            &params.bond_amount,
        );

        let dispute_id = Self::next_dispute_id(&env);
        let created_at = env.ledger().timestamp();
        let deadline = created_at
            .checked_add(params.timeout_secs)
            .ok_or(DisputeError::InvalidTimeout)?;

        let dispute = Dispute {
            id: dispute_id,
            mission_id: params.mission_id,
            hunter: hunter.clone(),
            respondent: params.respondent.clone(),
            arbiter: params.arbiter,
            bond_token: params.bond_token,
            hunter_bond: params.bond_amount,
            respondent_bond: 0,
            reason_cid: params.reason_cid,
            created_at,
            deadline,
            status: DisputeStatus::Open,
        };

        Self::store_dispute(&env, &dispute);
        env.storage().persistent().set(&open_key, &dispute_id);
        env.storage()
            .persistent()
            .extend_ttl(&open_key, TTL_LEDGERS, TTL_LEDGERS);

        DisputeCreatedEvent {
            dispute_id,
            mission_id: params.mission_id,
            hunter,
            respondent: params.respondent,
        }
        .publish(&env);

        BondStakedEvent {
            dispute_id,
            staker: dispute.hunter,
            amount: params.bond_amount,
        }
        .publish(&env);

        Ok(dispute_id)
    }

    /// Stake additional bond as the hunter or a counter-bond as the respondent.
    pub fn stake_bond(
        env: Env,
        dispute_id: u64,
        staker: Address,
        amount: i128,
    ) -> Result<(), DisputeError> {
        staker.require_auth();

        if amount <= 0 {
            return Err(DisputeError::InvalidAmount);
        }

        let mut dispute = Self::get_dispute(env.clone(), dispute_id)?;
        Self::require_open(&dispute)?;
        Self::require_before_deadline(&env, &dispute)?;

        if staker == dispute.hunter {
            dispute.hunter_bond = dispute
                .hunter_bond
                .checked_add(amount)
                .ok_or(DisputeError::InvalidAmount)?;
        } else if staker == dispute.respondent {
            dispute.respondent_bond = dispute
                .respondent_bond
                .checked_add(amount)
                .ok_or(DisputeError::InvalidAmount)?;
        } else {
            return Err(DisputeError::NotAuthorized);
        }

        token::Client::new(&env, &dispute.bond_token).transfer(
            &staker,
            env.current_contract_address(),
            &amount,
        );

        Self::store_dispute(&env, &dispute);

        BondStakedEvent {
            dispute_id,
            staker,
            amount,
        }
        .publish(&env);

        Ok(())
    }

    /// Arbiter (if set at create) awards the full bond pot to hunter or respondent.
    pub fn resolve_by_arbiter(
        env: Env,
        dispute_id: u64,
        arbiter: Address,
        hunter_wins: bool,
    ) -> Result<(), DisputeError> {
        arbiter.require_auth();

        let mut dispute = Self::get_dispute(env.clone(), dispute_id)?;
        Self::require_open(&dispute)?;
        Self::require_before_deadline(&env, &dispute)?;

        let named = dispute.arbiter.as_ref().ok_or(DisputeError::NoArbiter)?;
        if *named != arbiter {
            return Err(DisputeError::NotAuthorized);
        }

        let winner = if hunter_wins {
            dispute.hunter.clone()
        } else {
            dispute.respondent.clone()
        };
        Self::payout_pot(&env, &dispute, &winner)?;

        dispute.hunter_bond = 0;
        dispute.respondent_bond = 0;
        dispute.status = if hunter_wins {
            DisputeStatus::ResolvedHunter
        } else {
            DisputeStatus::ResolvedRespondent
        };

        Self::clear_open_index(&env, &dispute);
        Self::store_dispute(&env, &dispute);

        DisputeResolvedEvent {
            dispute_id,
            winner,
            hunter_wins,
        }
        .publish(&env);

        Ok(())
    }

    /// After the timelock, return each party's bond. Anyone may invoke.
    pub fn timeout_release(env: Env, dispute_id: u64) -> Result<(), DisputeError> {
        let mut dispute = Self::get_dispute(env.clone(), dispute_id)?;
        Self::require_open(&dispute)?;

        if env.ledger().timestamp() < dispute.deadline {
            return Err(DisputeError::TimeoutNotReached);
        }

        Self::refund_bonds(&env, &dispute)?;

        let hunter = dispute.hunter.clone();
        dispute.hunter_bond = 0;
        dispute.respondent_bond = 0;
        dispute.status = DisputeStatus::TimedOut;

        Self::clear_open_index(&env, &dispute);
        Self::store_dispute(&env, &dispute);

        DisputeTimeoutEvent { dispute_id, hunter }.publish(&env);

        Ok(())
    }

    pub fn get_dispute(env: Env, dispute_id: u64) -> Result<Dispute, DisputeError> {
        env.storage()
            .persistent()
            .get(&DataKey::Dispute(dispute_id))
            .ok_or(DisputeError::DisputeNotFound)
    }

    pub fn get_dispute_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0)
    }

    /// Open dispute id for a (mission, hunter) pair, if any.
    pub fn get_open_dispute(env: Env, mission_id: u64, hunter: Address) -> Option<u64> {
        env.storage()
            .persistent()
            .get(&DataKey::OpenDispute(mission_id, hunter))
    }

    fn next_dispute_id(env: &Env) -> u64 {
        let mut count: u64 = env
            .storage()
            .instance()
            .get(&DataKey::DisputeCount)
            .unwrap_or(0);
        count += 1;
        env.storage().instance().set(&DataKey::DisputeCount, &count);
        count
    }

    fn store_dispute(env: &Env, dispute: &Dispute) {
        let key = DataKey::Dispute(dispute.id);
        env.storage().persistent().set(&key, dispute);
        env.storage()
            .persistent()
            .extend_ttl(&key, TTL_LEDGERS, TTL_LEDGERS);
    }

    fn require_open(dispute: &Dispute) -> Result<(), DisputeError> {
        if dispute.status != DisputeStatus::Open {
            return Err(DisputeError::InvalidState);
        }
        Ok(())
    }

    fn require_before_deadline(env: &Env, dispute: &Dispute) -> Result<(), DisputeError> {
        if env.ledger().timestamp() >= dispute.deadline {
            return Err(DisputeError::TimeoutReached);
        }
        Ok(())
    }

    fn payout_pot(env: &Env, dispute: &Dispute, winner: &Address) -> Result<(), DisputeError> {
        let total = dispute
            .hunter_bond
            .checked_add(dispute.respondent_bond)
            .ok_or(DisputeError::InvalidAmount)?;
        if total > 0 {
            let contract = env.current_contract_address();
            token::Client::new(env, &dispute.bond_token).transfer(&contract, winner, &total);
        }
        Ok(())
    }

    fn refund_bonds(env: &Env, dispute: &Dispute) -> Result<(), DisputeError> {
        let token_client = token::Client::new(env, &dispute.bond_token);
        let contract = env.current_contract_address();
        if dispute.hunter_bond > 0 {
            token_client.transfer(&contract, &dispute.hunter, &dispute.hunter_bond);
        }
        if dispute.respondent_bond > 0 {
            token_client.transfer(&contract, &dispute.respondent, &dispute.respondent_bond);
        }
        Ok(())
    }

    fn clear_open_index(env: &Env, dispute: &Dispute) {
        env.storage().persistent().remove(&DataKey::OpenDispute(
            dispute.mission_id,
            dispute.hunter.clone(),
        ));
    }
}

mod test;
