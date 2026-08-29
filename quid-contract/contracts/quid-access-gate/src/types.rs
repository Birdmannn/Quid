use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Admin address
    Admin,
    /// Total number of rules configured
    RuleCount,
    /// Access gate rule by ID
    Rule(u64),
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GateType {
    /// Require minimum token balance
    TokenBalance,
    /// Require NFT ownership (min_amount = number of NFTs)
    NftOwnership,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GateRule {
    /// Unique rule ID
    pub id: u64,
    /// Type of gate check
    pub gate_type: GateType,
    /// Token or NFT contract address
    pub token_address: Address,
    /// Minimum amount required (balance for tokens, count for NFTs)
    pub min_amount: i128,
    /// Whether the rule is active
    pub active: bool,
    /// Timestamp when rule was created
    pub created_at: u64,
}
