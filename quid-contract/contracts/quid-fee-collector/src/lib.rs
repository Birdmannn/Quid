#![no_std]
use soroban_sdk::{contract, contractevent, contractimpl, token, Address, Env, Vec};

mod error;
mod types;

use error::FeeError;
use types::{DataKey, TokenBalance};

/// 100% expressed in basis points.
pub const MAX_FEE_BPS: u32 = 10_000;

const BALANCE_TTL_LEDGERS: u32 = 5_184_000;

#[contractevent(topics = ["fee", "collected"])]
pub struct FeeCollectedEvent {
    pub token: Address,
    pub from: Address,
    pub amount: i128,
}

#[contractevent(topics = ["fee", "withdrawn"])]
pub struct FeeWithdrawnEvent {
    pub token: Address,
    pub to: Address,
    pub amount: i128,
}

#[contractevent(topics = ["fee", "bps"], data_format = "single-value")]
pub struct FeeBpsUpdatedEvent {
    pub fee_bps: u32,
}

/// Protocol fee vault.
///
/// Receives a configurable cut of mission value from `quid-store` (or any other
/// caller), tracks the undrawn balance per token, and lets the admin withdraw.
#[contract]
pub struct QuidFeeCollectorContract;

#[contractimpl]
impl QuidFeeCollectorContract {
    // -------------------------------------------------------------------------
    // Admin bootstrap
    // -------------------------------------------------------------------------

    pub fn initialize(env: Env, admin: Address, fee_bps: u32) -> Result<(), FeeError> {
        admin.require_auth();

        if env.storage().instance().has(&DataKey::Admin) {
            return Err(FeeError::AlreadyInitialized);
        }
        if fee_bps > MAX_FEE_BPS {
            return Err(FeeError::InvalidFeeBps);
        }

        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, FeeError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(FeeError::NotInitialized)
    }

    /// Hand the vault over to a new admin. Current admin only.
    pub fn set_admin(env: Env, caller: Address, new_admin: Address) -> Result<(), FeeError> {
        Self::require_admin(&env, &caller)?;
        env.storage().instance().set(&DataKey::Admin, &new_admin);
        Ok(())
    }

    // -------------------------------------------------------------------------
    // Fee rate
    // -------------------------------------------------------------------------

    /// Set the protocol fee rate in basis points. Admin only.
    pub fn set_fee_bps(env: Env, caller: Address, fee_bps: u32) -> Result<(), FeeError> {
        Self::require_admin(&env, &caller)?;

        if fee_bps > MAX_FEE_BPS {
            return Err(FeeError::InvalidFeeBps);
        }

        env.storage().instance().set(&DataKey::FeeBps, &fee_bps);

        FeeBpsUpdatedEvent { fee_bps }.publish(&env);

        Ok(())
    }

    pub fn get_fee_bps(env: Env) -> u32 {
        env.storage().instance().get(&DataKey::FeeBps).unwrap_or(0)
    }

    /// Fee owed on `gross_amount` at the current rate, rounded down.
    pub fn compute_fee(env: Env, gross_amount: i128) -> Result<i128, FeeError> {
        if gross_amount < 0 {
            return Err(FeeError::InvalidAmount);
        }

        let fee_bps = Self::get_fee_bps(env) as i128;

        gross_amount
            .checked_mul(fee_bps)
            .map(|scaled| scaled / MAX_FEE_BPS as i128)
            .ok_or(FeeError::Overflow)
    }

    // -------------------------------------------------------------------------
    // Collection
    // -------------------------------------------------------------------------

    /// Charge the current fee rate on `gross_amount` and pull it from `from`.
    ///
    /// This is the entrypoint `quid-store` uses: it passes the value the fee is
    /// assessed on and the vault decides the cut, so the rate lives in one place.
    /// Returns the fee actually collected (zero when the rate is zero).
    pub fn collect_fee(
        env: Env,
        from: Address,
        token: Address,
        gross_amount: i128,
    ) -> Result<i128, FeeError> {
        let fee = Self::compute_fee(env.clone(), gross_amount)?;

        if fee > 0 {
            Self::deposit_fee(env, from, token, fee)?;
        }

        Ok(fee)
    }

    /// Pull an exact fee amount from `from` into the vault.
    pub fn deposit_fee(
        env: Env,
        from: Address,
        token: Address,
        amount: i128,
    ) -> Result<(), FeeError> {
        from.require_auth();

        if amount <= 0 {
            return Err(FeeError::InvalidAmount);
        }

        token::Client::new(&env, &token).transfer(&from, env.current_contract_address(), &amount);

        let new_balance = Self::get_balance(env.clone(), token.clone())
            .checked_add(amount)
            .ok_or(FeeError::Overflow)?;
        Self::store_balance(&env, &token, new_balance);
        Self::register_token(&env, &token);

        FeeCollectedEvent {
            token,
            from,
            amount,
        }
        .publish(&env);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Withdrawal
    // -------------------------------------------------------------------------

    /// Withdraw collected fees to `to`. Admin only.
    pub fn withdraw_fees(
        env: Env,
        caller: Address,
        token: Address,
        to: Address,
        amount: i128,
    ) -> Result<(), FeeError> {
        Self::require_admin(&env, &caller)?;

        if amount <= 0 {
            return Err(FeeError::InvalidAmount);
        }

        let balance = Self::get_balance(env.clone(), token.clone());
        if amount > balance {
            return Err(FeeError::InsufficientBalance);
        }

        token::Client::new(&env, &token).transfer(&env.current_contract_address(), &to, &amount);

        Self::store_balance(&env, &token, balance - amount);

        FeeWithdrawnEvent { token, to, amount }.publish(&env);

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Views
    // -------------------------------------------------------------------------

    pub fn get_balance(env: Env, token: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::Balance(token))
            .unwrap_or(0)
    }

    /// Undrawn balance for every token this vault has ever collected in.
    pub fn get_balances(env: Env) -> Vec<TokenBalance> {
        let mut balances = Vec::new(&env);

        for token in Self::get_tokens(env.clone()).iter() {
            let amount = Self::get_balance(env.clone(), token.clone());
            balances.push_back(TokenBalance { token, amount });
        }

        balances
    }

    pub fn get_tokens(env: Env) -> Vec<Address> {
        env.storage()
            .instance()
            .get(&DataKey::Tokens)
            .unwrap_or_else(|| Vec::new(&env))
    }

    // -------------------------------------------------------------------------
    // Internals
    // -------------------------------------------------------------------------

    fn require_admin(env: &Env, caller: &Address) -> Result<(), FeeError> {
        caller.require_auth();

        let admin = Self::get_admin(env.clone())?;
        if *caller != admin {
            return Err(FeeError::NotAuthorized);
        }

        Ok(())
    }

    fn store_balance(env: &Env, token: &Address, amount: i128) {
        let key = DataKey::Balance(token.clone());
        env.storage().persistent().set(&key, &amount);
        env.storage()
            .persistent()
            .extend_ttl(&key, BALANCE_TTL_LEDGERS, BALANCE_TTL_LEDGERS);
    }

    fn register_token(env: &Env, token: &Address) {
        let mut tokens = Self::get_tokens(env.clone());
        if !tokens.contains(token) {
            tokens.push_back(token.clone());
            env.storage().instance().set(&DataKey::Tokens, &tokens);
        }
    }
}

mod test;
