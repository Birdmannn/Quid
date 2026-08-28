use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum DisputeError {
    NotAuthorized = 1,
    InvalidAmount = 2,
    DisputeNotFound = 3,
    InvalidState = 4,
    AlreadyDisputed = 5,
    TimeoutNotReached = 6,
    TimeoutReached = 7,
    NoArbiter = 8,
    InvalidTimeout = 9,
    InvalidParties = 10,
}
