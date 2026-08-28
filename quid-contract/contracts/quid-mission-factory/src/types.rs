use soroban_sdk::{contracttype, Address, String};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reward {
    pub reward_token: Address,
    pub reward_amount: i128,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MinAsset {
    pub min_asset_token: Option<Address>,
    pub min_asset_amount: i128,
}

/// The reusable defaults supplied by an ecosystem partner.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateConfig {
    pub store: Address,
    pub name: String,
    pub title: String,
    pub description_cid: String,
    pub reward: Reward,
    pub max_participants: u32,
    pub min_asset: MinAsset,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MissionTemplate {
    pub id: u64,
    pub store: Address,
    pub name: String,
    pub title: String,
    pub description_cid: String,
    pub reward: Reward,
    pub max_participants: u32,
    pub min_asset: MinAsset,
    pub created_at: u64,
}

#[contracttype]
pub(crate) enum DataKey {
    Admin,
    TemplateCount,
    Template(u64),
}
