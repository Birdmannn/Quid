use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum LeaderboardError {
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Caller is not the admin or authorized recorder.
    NotAuthorized = 2,
    /// No active epoch exists when one is required.
    NoActiveEpoch = 3,
    /// An epoch is already active when one must not be.
    EpochAlreadyActive = 4,
    /// The requested epoch ID does not exist.
    EpochNotFound = 5,
    /// n = 0 or exceeds the entry count for the epoch.
    InvalidN = 6,
    /// Score delta would overflow an i64.
    ScoreOverflow = 7,
    /// Contract has not been initialized yet.
    NotInitialized = 8,
}
