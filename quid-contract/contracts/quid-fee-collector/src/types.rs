use soroban_sdk::{contracttype, Address};

/// Collected (undrawn) fee balance held for a single token.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TokenBalance {
    pub token: Address,
    pub amount: i128,
}

#[contracttype]
pub enum DataKey {
    Admin,
    /// Protocol fee rate in basis points (1 bps = 0.01%).
    FeeBps,
    /// Undrawn balance collected for one token.
    Balance(Address),
    /// Every token this collector has ever received a fee in.
    Tokens,
}
