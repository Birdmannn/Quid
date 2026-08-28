use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum FeeError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    InvalidFeeBps = 4,
    InvalidAmount = 5,
    InsufficientBalance = 6,
    Overflow = 7,
}
