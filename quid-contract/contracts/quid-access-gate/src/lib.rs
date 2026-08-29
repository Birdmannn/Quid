#![no_std]

use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

mod error;
mod types;

pub use error::AccessGateError;
pub use types::{DataKey, GateRule, GateType};

const RULE_TTL_LEDGERS: u32 = 5_184_000; // ~1 year
const MAX_RULES: u64 = 1000;

#[contract]
pub struct QuidAccessGateContract;

#[contractimpl]
impl QuidAccessGateContract {
    /// Initialize the contract with an admin address
    pub fn initialize(env: Env, admin: Address) -> Result<(), AccessGateError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AccessGateError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::RuleCount, &0u64);
        Ok(())
    }

    /// Get the admin address
    pub fn get_admin(env: Env) -> Result<Address, AccessGateError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(AccessGateError::NotInitialized)
    }

    /// Configure a new access gate rule
    /// Returns the rule ID
    pub fn configure_rule(
        env: Env,
        gate_type: GateType,
        token_address: Address,
        min_amount: i128,
    ) -> Result<u64, AccessGateError> {
        let admin = Self::get_admin(env.clone())?;
        admin.require_auth();

        // Validate parameters
        if min_amount <= 0 {
            return Err(AccessGateError::InvalidAmount);
        }

        // Check rule limit
        let count = Self::get_rule_count(env.clone());
        if count >= MAX_RULES {
            return Err(AccessGateError::RuleLimitReached);
        }

        let rule_id = count + 1;
        let rule = GateRule {
            id: rule_id,
            gate_type,
            token_address,
            min_amount,
            active: true,
            created_at: env.ledger().timestamp(),
        };

        let key = DataKey::Rule(rule_id);
        env.storage().persistent().set(&key, &rule);
        env.storage()
            .persistent()
            .extend_ttl(&key, RULE_TTL_LEDGERS, RULE_TTL_LEDGERS);

        env.storage().instance().set(&DataKey::RuleCount, &rule_id);

        Ok(rule_id)
    }

    /// Get a specific rule by ID
    pub fn get_rule(env: Env, rule_id: u64) -> Result<GateRule, AccessGateError> {
        let key = DataKey::Rule(rule_id);
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(AccessGateError::RuleNotFound)
    }

    /// Get the total number of rules configured
    pub fn get_rule_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::RuleCount)
            .unwrap_or(0)
    }

    /// Deactivate a rule (soft delete)
    pub fn deactivate_rule(env: Env, rule_id: u64) -> Result<(), AccessGateError> {
        let admin = Self::get_admin(env.clone())?;
        admin.require_auth();

        let mut rule = Self::get_rule(env.clone(), rule_id)?;
        rule.active = false;

        let key = DataKey::Rule(rule_id);
        env.storage().persistent().set(&key, &rule);
        env.storage()
            .persistent()
            .extend_ttl(&key, RULE_TTL_LEDGERS, RULE_TTL_LEDGERS);

        Ok(())
    }

    /// Check if an address passes a specific access gate rule
    /// Returns true if the address meets the requirements, false otherwise
    pub fn check_access(env: Env, address: Address, rule_id: u64) -> Result<bool, AccessGateError> {
        let rule = Self::get_rule(env.clone(), rule_id)?;

        // Inactive rules always fail
        if !rule.active {
            return Ok(false);
        }

        // Check based on gate type
        match rule.gate_type {
            GateType::TokenBalance => {
                Self::check_token_balance(&env, &address, &rule.token_address, rule.min_amount)
            }
            GateType::NftOwnership => {
                Self::check_nft_ownership(&env, &address, &rule.token_address, rule.min_amount)
            }
        }
    }

    /// Check multiple rules at once
    /// Returns true only if ALL rules pass
    pub fn check_multiple_access(
        env: Env,
        address: Address,
        rule_ids: Vec<u64>,
    ) -> Result<bool, AccessGateError> {
        for rule_id in rule_ids.iter() {
            if !Self::check_access(env.clone(), address.clone(), rule_id)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Internal: Check if address has sufficient token balance
    fn check_token_balance(
        env: &Env,
        address: &Address,
        token_address: &Address,
        min_amount: i128,
    ) -> Result<bool, AccessGateError> {
        use soroban_sdk::token;

        let token_client = token::TokenClient::new(env, token_address);
        let balance = token_client.balance(address);

        Ok(balance >= min_amount)
    }

    /// Internal: Check if address owns NFT(s)
    /// For NFTs, min_amount represents the minimum number of NFTs to own
    fn check_nft_ownership(
        env: &Env,
        address: &Address,
        nft_address: &Address,
        min_amount: i128,
    ) -> Result<bool, AccessGateError> {
        use soroban_sdk::token;

        // NFT contracts implement the token interface
        // Balance represents number of NFTs owned
        let nft_client = token::TokenClient::new(env, nft_address);
        let balance = nft_client.balance(address);

        Ok(balance >= min_amount)
    }
}

#[cfg(test)]
mod test;
