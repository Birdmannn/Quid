#![no_std]

use soroban_sdk::{contract, contractimpl, token, Address, Env};

mod error;
mod types;

pub use error::StakingError;
use types::DataKey;

const STAKE_TTL_LEDGERS: u32 = 5_184_000;

#[contract]
pub struct QuidStakingContract;

#[contractimpl]
impl QuidStakingContract {
    pub fn initialize(env: Env, admin: Address, treasury: Address) -> Result<(), StakingError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(StakingError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::Treasury, &treasury);
        Ok(())
    }

    pub fn set_locker(
        env: Env,
        caller: Address,
        locker: Address,
        enabled: bool,
    ) -> Result<(), StakingError> {
        Self::require_admin(&env, &caller)?;
        env.storage()
            .persistent()
            .set(&DataKey::Locker(locker.clone()), &enabled);
        env.storage().persistent().extend_ttl(
            &DataKey::Locker(locker),
            STAKE_TTL_LEDGERS,
            STAKE_TTL_LEDGERS,
        );
        Ok(())
    }

    pub fn deposit(
        env: Env,
        hunter: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<(), StakingError> {
        hunter.require_auth();
        Self::validate_amount(amount)?;

        token::Client::new(&env, &token_address).transfer(
            &hunter,
            env.current_contract_address(),
            &amount,
        );
        let balance = Self::balance(&env, &hunter, &token_address);
        Self::store_balance(
            &env,
            &hunter,
            &token_address,
            balance
                .checked_add(amount)
                .ok_or(StakingError::InvalidAmount)?,
        );
        Ok(())
    }

    pub fn withdraw(
        env: Env,
        hunter: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<(), StakingError> {
        hunter.require_auth();
        Self::validate_amount(amount)?;
        let balance = Self::balance(&env, &hunter, &token_address);
        let locked = Self::locked_balance(&env, &hunter, &token_address);
        if balance
            .checked_sub(locked)
            .ok_or(StakingError::InvalidAmount)?
            < amount
        {
            return Err(StakingError::InsufficientAvailableBalance);
        }

        token::Client::new(&env, &token_address).transfer(
            &env.current_contract_address(),
            &hunter,
            &amount,
        );
        Self::store_balance(&env, &hunter, &token_address, balance - amount);
        Ok(())
    }

    pub fn lock_for_mission(
        env: Env,
        locker: Address,
        mission_id: u64,
        hunter: Address,
        token_address: Address,
        amount: i128,
    ) -> Result<(), StakingError> {
        locker.require_auth();
        Self::require_locker(&env, &locker)?;
        Self::validate_amount(amount)?;

        let lock_key =
            DataKey::MissionLock(locker, mission_id, hunter.clone(), token_address.clone());
        if env.storage().persistent().has(&lock_key) {
            return Err(StakingError::LockAlreadyExists);
        }
        let available = Self::balance(&env, &hunter, &token_address)
            .checked_sub(Self::locked_balance(&env, &hunter, &token_address))
            .ok_or(StakingError::InvalidAmount)?;
        if available < amount {
            return Err(StakingError::InsufficientAvailableBalance);
        }

        env.storage().persistent().set(&lock_key, &amount);
        env.storage()
            .persistent()
            .extend_ttl(&lock_key, STAKE_TTL_LEDGERS, STAKE_TTL_LEDGERS);
        Self::store_locked_balance(
            &env,
            &hunter,
            &token_address,
            Self::locked_balance(&env, &hunter, &token_address) + amount,
        );
        Ok(())
    }

    pub fn unlock_for_mission(
        env: Env,
        locker: Address,
        mission_id: u64,
        hunter: Address,
        token_address: Address,
    ) -> Result<(), StakingError> {
        locker.require_auth();
        Self::require_locker(&env, &locker)?;
        Self::release_lock(&env, locker, mission_id, hunter, token_address).map(|_| ())
    }

    pub fn slash_for_mission(
        env: Env,
        locker: Address,
        mission_id: u64,
        hunter: Address,
        token_address: Address,
    ) -> Result<(), StakingError> {
        locker.require_auth();
        Self::require_locker(&env, &locker)?;
        let amount = Self::release_lock(
            &env,
            locker,
            mission_id,
            hunter.clone(),
            token_address.clone(),
        )?;
        let balance = Self::balance(&env, &hunter, &token_address);
        let treasury: Address = env
            .storage()
            .instance()
            .get(&DataKey::Treasury)
            .ok_or(StakingError::NotInitialized)?;
        token::Client::new(&env, &token_address).transfer(
            &env.current_contract_address(),
            &treasury,
            &amount,
        );
        Self::store_balance(&env, &hunter, &token_address, balance - amount);
        Ok(())
    }

    pub fn get_balance(env: Env, hunter: Address, token_address: Address) -> i128 {
        Self::balance(&env, &hunter, &token_address)
    }

    pub fn get_locked_balance(env: Env, hunter: Address, token_address: Address) -> i128 {
        Self::locked_balance(&env, &hunter, &token_address)
    }

    fn release_lock(
        env: &Env,
        locker: Address,
        mission_id: u64,
        hunter: Address,
        token_address: Address,
    ) -> Result<i128, StakingError> {
        let key = DataKey::MissionLock(locker, mission_id, hunter.clone(), token_address.clone());
        let amount: i128 = env
            .storage()
            .persistent()
            .get(&key)
            .ok_or(StakingError::LockNotFound)?;
        let locked = Self::locked_balance(env, &hunter, &token_address);
        env.storage().persistent().remove(&key);
        Self::store_locked_balance(env, &hunter, &token_address, locked - amount);
        Ok(amount)
    }

    fn require_admin(env: &Env, caller: &Address) -> Result<(), StakingError> {
        caller.require_auth();
        let admin: Address = env
            .storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(StakingError::NotInitialized)?;
        if *caller != admin {
            return Err(StakingError::NotAuthorized);
        }
        Ok(())
    }

    fn require_locker(env: &Env, locker: &Address) -> Result<(), StakingError> {
        if !env
            .storage()
            .persistent()
            .get(&DataKey::Locker(locker.clone()))
            .unwrap_or(false)
        {
            return Err(StakingError::NotAuthorized);
        }
        Ok(())
    }

    fn validate_amount(amount: i128) -> Result<(), StakingError> {
        if amount <= 0 {
            return Err(StakingError::InvalidAmount);
        }
        Ok(())
    }

    fn balance(env: &Env, hunter: &Address, token_address: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(hunter.clone(), token_address.clone()))
            .unwrap_or(0)
    }

    fn locked_balance(env: &Env, hunter: &Address, token_address: &Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Locked(hunter.clone(), token_address.clone()))
            .unwrap_or(0)
    }

    fn store_balance(env: &Env, hunter: &Address, token_address: &Address, amount: i128) {
        let key = DataKey::Balance(hunter.clone(), token_address.clone());
        env.storage().persistent().set(&key, &amount);
        env.storage()
            .persistent()
            .extend_ttl(&key, STAKE_TTL_LEDGERS, STAKE_TTL_LEDGERS);
    }

    fn store_locked_balance(env: &Env, hunter: &Address, token_address: &Address, amount: i128) {
        let key = DataKey::Locked(hunter.clone(), token_address.clone());
        env.storage().persistent().set(&key, &amount);
        env.storage()
            .persistent()
            .extend_ttl(&key, STAKE_TTL_LEDGERS, STAKE_TTL_LEDGERS);
    }
}

#[cfg(test)]
mod test;
