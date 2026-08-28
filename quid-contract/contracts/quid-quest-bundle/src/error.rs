use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum QuestBundleError {
    NotAuthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    BundleNotFound = 4,
    /// The bundle is not `Active` (already cancelled).
    InvalidState = 5,
    InvalidAmount = 6,
    InvalidInput = 7,
    /// The mission id is already part of this bundle.
    MissionAlreadyAdded = 8,
    /// The mission id was never added to this bundle.
    MissionNotInBundle = 9,
    /// A bundle with no missions has nothing to complete.
    BundleEmpty = 10,
    /// This (bundle, hunter, mission) completion was already reported.
    CompletionAlreadyRecorded = 11,
    /// The hunter still owes at least one mission in the bundle.
    MissionsIncomplete = 12,
    /// This hunter already took their completion bonus.
    AlreadyClaimed = 13,
    /// Every bonus slot the founder funded has been claimed.
    NoClaimsRemaining = 14,
    /// Escrow holds less than the bonus owed - refuse rather than half-pay.
    InsufficientEscrow = 15,
    Overflow = 16,
}
