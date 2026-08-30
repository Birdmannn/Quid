use soroban_sdk::{contracttype, Address};

#[contracttype]
pub enum DataKey {
    Admin,
    Treasury,
    Locker(Address),
    Balance(Address, Address),
    Locked(Address, Address),
    MissionLock(Address, u64, Address, Address),
}
