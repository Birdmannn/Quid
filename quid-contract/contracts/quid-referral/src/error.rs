use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ReferralError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    InvalidRewardBps = 4,
    InvalidAmount = 5,
    InvalidInput = 6,
    SelfReferral = 7,
    AlreadyRegistered = 8,
    ReferralNotFound = 9,
    PayoutAlreadyRecorded = 10,
    NothingToClaim = 11,
    InsufficientFunds = 12,
    Overflow = 13,
}
