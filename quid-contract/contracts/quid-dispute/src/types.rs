use soroban_sdk::{contracttype, Address, String};

#[derive(Clone, Debug, Default, PartialEq, Eq, Copy)]
#[contracttype]
pub enum DisputeStatus {
    #[default]
    Open,
    ResolvedHunter,
    ResolvedRespondent,
    TimedOut,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Dispute {
    pub id: u64,
    pub mission_id: u64,
    pub hunter: Address,
    pub respondent: Address,
    pub arbiter: Option<Address>,
    pub bond_token: Address,
    pub hunter_bond: i128,
    pub respondent_bond: i128,
    pub reason_cid: String,
    pub created_at: u64,
    pub deadline: u64,
    pub status: DisputeStatus,
}

/// Arguments for opening a dispute. Hunter bond is staked on create.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreateDisputeParams {
    pub mission_id: u64,
    pub respondent: Address,
    pub arbiter: Option<Address>,
    pub bond_token: Address,
    pub bond_amount: i128,
    pub reason_cid: String,
    pub timeout_secs: u64,
}

#[contracttype]
pub enum DataKey {
    Dispute(u64),
    DisputeCount,
    /// Maps (mission_id, hunter) → open dispute id.
    OpenDispute(u64, Address),
}
