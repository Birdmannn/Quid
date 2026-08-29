#![no_std]

use soroban_sdk::{
    contract, contractclient, contractevent, contractimpl, Address, Env, String, Vec,
};

mod error;
mod types;

pub use error::FactoryError;
use types::DataKey;
pub use types::{MinAsset, MissionTemplate, Reward, TemplateConfig};

const TEMPLATE_TTL_LEDGERS: u32 = 5_184_000;
const MAX_TEMPLATES: u64 = 100;
const MAX_TEXT_LEN: u32 = 256;

/// The subset of `quid-store` used by the factory. Keeping this as an interface
/// avoids embedding the store contract in the factory Wasm.
#[contractclient(name = "StoreClient")]
pub trait Store {
    fn create_mission(
        env: Env,
        owner: Address,
        title: String,
        description_cid: String,
        reward: Reward,
        max_participants: u32,
        min_asset: MinAsset,
    ) -> u64;
}

#[contractevent(topics = ["template", "new"])]
pub struct TemplateRegisteredEvent {
    pub template_id: u64,
    pub store: Address,
}

#[contractevent(topics = ["mission", "from_tpl"])]
pub struct MissionFromTemplateEvent {
    pub template_id: u64,
    pub mission_id: u64,
    pub owner: Address,
    pub store: Address,
}

#[contract]
pub struct QuidMissionFactoryContract;

#[contractimpl]
impl QuidMissionFactoryContract {
    /// Bootstrap the template registry. Callable once; `admin` must authorize.
    pub fn initialize(env: Env, admin: Address) -> Result<(), FactoryError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(FactoryError::AlreadyInitialized);
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        Ok(())
    }

    pub fn get_admin(env: Env) -> Result<Address, FactoryError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin)
            .ok_or(FactoryError::NotInitialized)
    }

    /// Register an immutable, bounded mission template. Admin only.
    pub fn register_template(env: Env, config: TemplateConfig) -> Result<u64, FactoryError> {
        let admin = Self::get_admin(env.clone())?;
        admin.require_auth();
        Self::validate_template(&config)?;

        let count = Self::template_count(env.clone());
        if count >= MAX_TEMPLATES {
            return Err(FactoryError::TemplateLimitReached);
        }
        let template_id = count + 1;
        let template = MissionTemplate {
            id: template_id,
            store: config.store,
            name: config.name,
            title: config.title,
            description_cid: config.description_cid,
            reward: config.reward,
            max_participants: config.max_participants,
            min_asset: config.min_asset,
            created_at: env.ledger().timestamp(),
        };
        let key = DataKey::Template(template_id);
        env.storage().persistent().set(&key, &template);
        env.storage()
            .persistent()
            .extend_ttl(&key, TEMPLATE_TTL_LEDGERS, TEMPLATE_TTL_LEDGERS);
        env.storage()
            .instance()
            .set(&DataKey::TemplateCount, &template_id);

        TemplateRegisteredEvent {
            template_id,
            store: template.store,
        }
        .publish(&env);
        Ok(template_id)
    }

    /// Create and fund a mission in the template's configured `quid-store`.
    /// The owner authorizes the whole call tree, including token escrow.
    pub fn create_from_template(
        env: Env,
        template_id: u64,
        owner: Address,
    ) -> Result<u64, FactoryError> {
        owner.require_auth();
        let template = Self::get_template(env.clone(), template_id)?;
        let store = template.store.clone();
        let mission_id = StoreClient::new(&env, &store).create_mission(
            &owner,
            &template.title,
            &template.description_cid,
            &template.reward,
            &template.max_participants,
            &template.min_asset,
        );

        MissionFromTemplateEvent {
            template_id,
            mission_id,
            owner,
            store,
        }
        .publish(&env);
        Ok(mission_id)
    }

    pub fn get_template(env: Env, template_id: u64) -> Result<MissionTemplate, FactoryError> {
        env.storage()
            .persistent()
            .get(&DataKey::Template(template_id))
            .ok_or(FactoryError::TemplateNotFound)
    }

    /// Return every registered template, oldest first (registry capped at 100).
    pub fn list_templates(env: Env) -> Vec<MissionTemplate> {
        let mut templates = Vec::new(&env);
        for id in 1..=Self::template_count(env.clone()) {
            if let Some(template) = env
                .storage()
                .persistent()
                .get::<_, MissionTemplate>(&DataKey::Template(id))
            {
                templates.push_back(template);
            }
        }
        templates
    }

    pub fn template_count(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::TemplateCount)
            .unwrap_or(0)
    }

    fn validate_template(config: &TemplateConfig) -> Result<(), FactoryError> {
        if config.name.is_empty()
            || config.name.len() > MAX_TEXT_LEN
            || config.title.is_empty()
            || config.title.len() > MAX_TEXT_LEN
            || config.description_cid.is_empty()
            || config.description_cid.len() > MAX_TEXT_LEN
            || config.reward.reward_amount <= 0
            || config.max_participants == 0
            || (config.min_asset.min_asset_token.is_some()
                && config.min_asset.min_asset_amount <= 0)
            || (config.min_asset.min_asset_token.is_none()
                && config.min_asset.min_asset_amount != 0)
            || config
                .reward
                .reward_amount
                .checked_mul(config.max_participants as i128)
                .is_none()
        {
            return Err(FactoryError::InvalidTemplate);
        }
        Ok(())
    }
}

#[cfg(test)]
mod test;
