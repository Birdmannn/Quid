use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum TreasuryError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    NotSigner = 3,
    InvalidSigners = 4,
    InvalidThreshold = 5,
    InvalidAmount = 6,
    InvalidExpiry = 7,
    ProposalNotFound = 8,
    ProposalExpired = 9,
    ProposalExecuted = 10,
    AlreadyApproved = 11,
    InsufficientApprovals = 12,
    Overflow = 13,
}
