use soroban_sdk::{contracttype, Address, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransferProposal {
    pub id: u64,
    pub fee_collector: Address,
    pub token: Address,
    pub to: Address,
    pub amount: i128,
    pub expires_at: u64,
    pub approvals: u32,
    pub executed: bool,
}

#[contracttype]
pub enum DataKey {
    Signers,
    Threshold,
    ProposalCount,
    Proposal(u64),
    Approval(u64, Address),
}

pub type Signers = Vec<Address>;
