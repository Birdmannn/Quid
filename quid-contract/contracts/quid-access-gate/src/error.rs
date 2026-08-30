use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccessGateError {
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Contract not initialized
    NotInitialized = 2,
    /// Rule not found
    RuleNotFound = 3,
    /// Invalid amount (must be positive)
    InvalidAmount = 4,
    /// Maximum number of rules reached
    RuleLimitReached = 5,
}
