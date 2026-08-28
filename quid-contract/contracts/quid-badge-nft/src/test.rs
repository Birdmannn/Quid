#![cfg(test)]

use crate::{
    error::BadgeError,
    types::{BadgeKind, BadgeSpec},
    QuidBadgeNftContract, QuidBadgeNftContractClient,
};
use soroban_sdk::{testutils::Address as _, Address, Env, String};

fn setup() -> (Env, QuidBadgeNftContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidBadgeNftContract, ());
    let client = QuidBadgeNftContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(
        &admin,
        &String::from_str(&env, "Quid Badges"),
        &String::from_str(&env, "QBADGE"),
        &String::from_str(&env, "ipfs://"),
    );

    (env, client, admin)
}

fn mission_badge(env: &Env, mission_id: u64, soulbound: bool) -> BadgeSpec {
    BadgeSpec {
        kind: BadgeKind::MissionComplete,
        reference: mission_id,
        metadata_cid: String::from_str(env, "QmBadgeMetadataCid"),
        soulbound,
    }
}

// -------------------------------------------------------------------------
// Admin bootstrap
// -------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin_and_collection() {
    let (env, client, admin) = setup();

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.name(), String::from_str(&env, "Quid Badges"));
    assert_eq!(client.symbol(), String::from_str(&env, "QBADGE"));
    assert_eq!(client.base_uri(), String::from_str(&env, "ipfs://"));
    assert_eq!(client.total_minted(), 0);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, client, _admin) = setup();
    let other = Address::generate(&env);

    assert_eq!(
        client.try_initialize(
            &other,
            &String::from_str(&env, "Copy"),
            &String::from_str(&env, "CPY"),
            &String::from_str(&env, "ipfs://"),
        ),
        Err(Ok(BadgeError::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_rejects_empty_symbol() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidBadgeNftContract, ());
    let client = QuidBadgeNftContractClient::new(&env, &contract_id);

    assert_eq!(
        client.try_initialize(
            &Address::generate(&env),
            &String::from_str(&env, "Quid Badges"),
            &String::from_str(&env, ""),
            &String::from_str(&env, "ipfs://"),
        ),
        Err(Ok(BadgeError::InvalidCollection))
    );
}

#[test]
fn test_uninitialized_contract_has_no_admin() {
    let env = Env::default();
    let contract_id = env.register(QuidBadgeNftContract, ());
    let client = QuidBadgeNftContractClient::new(&env, &contract_id);

    assert_eq!(client.try_get_admin(), Err(Ok(BadgeError::NotInitialized)));
    assert!(!client.is_minter(&Address::generate(&env)));
}

#[test]
fn test_set_admin_moves_role() {
    let (env, client, admin) = setup();
    let new_admin = Address::generate(&env);

    client.set_admin(&new_admin);

    assert_eq!(client.get_admin(), new_admin);
    assert!(client.is_minter(&new_admin));
    assert!(!client.is_minter(&admin));
}

// -------------------------------------------------------------------------
// Minter allow-list
// -------------------------------------------------------------------------

#[test]
fn test_admin_is_minter_by_default() {
    let (_env, client, admin) = setup();

    assert!(client.is_minter(&admin));
}

#[test]
fn test_add_and_remove_minter() {
    let (env, client, _admin) = setup();
    let store_hook = Address::generate(&env);

    assert!(!client.is_minter(&store_hook));

    client.add_minter(&store_hook);
    assert!(client.is_minter(&store_hook));

    client.remove_minter(&store_hook);
    assert!(!client.is_minter(&store_hook));
}

#[test]
fn test_add_minter_twice_fails() {
    let (env, client, _admin) = setup();
    let store_hook = Address::generate(&env);

    client.add_minter(&store_hook);

    assert_eq!(
        client.try_add_minter(&store_hook),
        Err(Ok(BadgeError::AlreadyMinter))
    );
}

#[test]
fn test_remove_unknown_minter_fails() {
    let (env, client, _admin) = setup();

    assert_eq!(
        client.try_remove_minter(&Address::generate(&env)),
        Err(Ok(BadgeError::MinterNotFound))
    );
}

// -------------------------------------------------------------------------
// Minting
// -------------------------------------------------------------------------

#[test]
fn test_admin_mints_badge_with_metadata_cid() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 42, true));

    assert_eq!(badge_id, 1);
    assert_eq!(client.total_minted(), 1);
    assert!(client.badge_exists(&badge_id));

    let badge = client.get_badge(&badge_id);
    assert_eq!(badge.id, 1);
    assert_eq!(badge.owner, hunter);
    assert_eq!(badge.minted_to, hunter);
    assert_eq!(badge.minted_by, admin);
    assert_eq!(badge.kind, BadgeKind::MissionComplete);
    assert_eq!(badge.reference, 42);
    assert_eq!(
        badge.metadata_cid,
        String::from_str(&env, "QmBadgeMetadataCid")
    );
    assert!(badge.soulbound);
    assert_eq!(badge.minted_at, env.ledger().timestamp());

    assert_eq!(client.owner_of(&badge_id), hunter);
    assert_eq!(
        client.metadata_cid(&badge_id),
        String::from_str(&env, "QmBadgeMetadataCid")
    );
}

#[test]
fn test_allow_listed_minter_can_mint() {
    let (env, client, _admin) = setup();
    let store_hook = Address::generate(&env);
    let hunter = Address::generate(&env);

    client.add_minter(&store_hook);
    let badge_id = client.mint_badge(&store_hook, &hunter, &mission_badge(&env, 7, true));

    assert_eq!(client.get_badge(&badge_id).minted_by, store_hook);
}

#[test]
fn test_mint_rejected_for_unauthorized_caller() {
    let (env, client, _admin) = setup();
    let stranger = Address::generate(&env);
    let hunter = Address::generate(&env);

    assert_eq!(
        client.try_mint_badge(&stranger, &hunter, &mission_badge(&env, 1, true)),
        Err(Ok(BadgeError::NotAuthorized))
    );
    assert_eq!(client.total_minted(), 0);
    assert_eq!(client.balance_of(&hunter), 0);
}

#[test]
fn test_mint_rejected_after_minter_revoked() {
    let (env, client, _admin) = setup();
    let store_hook = Address::generate(&env);
    let hunter = Address::generate(&env);

    client.add_minter(&store_hook);
    client.mint_badge(&store_hook, &hunter, &mission_badge(&env, 1, true));
    client.remove_minter(&store_hook);

    assert_eq!(
        client.try_mint_badge(&store_hook, &hunter, &mission_badge(&env, 2, true)),
        Err(Ok(BadgeError::NotAuthorized))
    );
}

#[test]
fn test_mint_requires_minter_authorization() {
    let env = Env::default();
    let contract_id = env.register(QuidBadgeNftContract, ());
    let client = QuidBadgeNftContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    env.mock_all_auths();
    client.initialize(
        &admin,
        &String::from_str(&env, "Quid Badges"),
        &String::from_str(&env, "QBADGE"),
        &String::from_str(&env, "ipfs://"),
    );

    // Drop the mocks: mint_badge must now fail on `minter.require_auth()`.
    env.set_auths(&[]);

    let hunter = Address::generate(&env);
    assert!(client
        .try_mint_badge(&admin, &hunter, &mission_badge(&env, 1, true))
        .is_err());
    assert_eq!(client.total_minted(), 0);
}

#[test]
fn test_duplicate_milestone_badge_rejected() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    client.mint_badge(&admin, &hunter, &mission_badge(&env, 42, true));

    assert!(client.has_claimed(&hunter, &BadgeKind::MissionComplete, &42));
    assert_eq!(
        client.try_mint_badge(&admin, &hunter, &mission_badge(&env, 42, true)),
        Err(Ok(BadgeError::AlreadyMinted))
    );
    assert_eq!(client.balance_of(&hunter), 1);
}

#[test]
fn test_same_reference_different_kind_is_allowed() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    client.mint_badge(&admin, &hunter, &mission_badge(&env, 3, true));
    client.mint_badge(
        &admin,
        &hunter,
        &BadgeSpec {
            kind: BadgeKind::ReputationTier,
            reference: 3,
            metadata_cid: String::from_str(&env, "QmTierThree"),
            soulbound: true,
        },
    );

    assert_eq!(client.balance_of(&hunter), 2);
}

#[test]
fn test_same_milestone_for_two_hunters_is_allowed() {
    let (env, client, admin) = setup();
    let first = Address::generate(&env);
    let second = Address::generate(&env);

    client.mint_badge(&admin, &first, &mission_badge(&env, 9, true));
    client.mint_badge(&admin, &second, &mission_badge(&env, 9, true));

    assert_eq!(client.balance_of(&first), 1);
    assert_eq!(client.balance_of(&second), 1);
    assert_eq!(client.total_minted(), 2);
}

#[test]
fn test_mint_rejects_empty_metadata_cid() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    let spec = BadgeSpec {
        kind: BadgeKind::MissionComplete,
        reference: 1,
        metadata_cid: String::from_str(&env, ""),
        soulbound: true,
    };

    assert_eq!(
        client.try_mint_badge(&admin, &hunter, &spec),
        Err(Ok(BadgeError::InvalidMetadata))
    );
}

#[test]
fn test_mint_rejects_oversized_metadata_cid() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    // 129 chars, one over MAX_CID_LEN.
    let long_cid = "Qm1234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567890123456789012345678901234567";
    assert_eq!(long_cid.len(), 129);

    let spec = BadgeSpec {
        kind: BadgeKind::Custom,
        reference: 1,
        metadata_cid: String::from_str(&env, long_cid),
        soulbound: false,
    };

    assert_eq!(
        client.try_mint_badge(&admin, &hunter, &spec),
        Err(Ok(BadgeError::InvalidMetadata))
    );
}

#[test]
fn test_get_missing_badge_fails() {
    let (_env, client, _admin) = setup();

    assert_eq!(
        client.try_get_badge(&404),
        Err(Ok(BadgeError::BadgeNotFound))
    );
    assert!(!client.badge_exists(&404));
}

// -------------------------------------------------------------------------
// list_by_owner
// -------------------------------------------------------------------------

#[test]
fn test_list_by_owner_returns_only_that_owners_badges() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);
    let other = Address::generate(&env);

    let first = client.mint_badge(&admin, &hunter, &mission_badge(&env, 1, true));
    let second = client.mint_badge(&admin, &hunter, &mission_badge(&env, 2, true));
    let third = client.mint_badge(&admin, &other, &mission_badge(&env, 3, true));

    let badges = client.list_by_owner(&hunter);
    assert_eq!(badges.len(), 2);
    assert_eq!(badges.get(0).unwrap().id, first);
    assert_eq!(badges.get(1).unwrap().id, second);
    assert_eq!(
        client.list_ids_by_owner(&hunter),
        soroban_sdk::vec![&env, first, second]
    );

    let other_badges = client.list_by_owner(&other);
    assert_eq!(other_badges.len(), 1);
    assert_eq!(other_badges.get(0).unwrap().id, third);
}

#[test]
fn test_list_by_owner_empty_for_unknown_address() {
    let (env, client, _admin) = setup();

    assert_eq!(client.list_by_owner(&Address::generate(&env)).len(), 0);
    assert_eq!(client.balance_of(&Address::generate(&env)), 0);
}

// -------------------------------------------------------------------------
// Transfer / burn
// -------------------------------------------------------------------------

#[test]
fn test_transfer_moves_transferable_badge() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);
    let collector = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 5, false));

    client.transfer(&hunter, &collector, &badge_id);

    assert_eq!(client.owner_of(&badge_id), collector);
    assert_eq!(client.balance_of(&hunter), 0);
    assert_eq!(client.balance_of(&collector), 1);
    assert_eq!(client.list_by_owner(&hunter).len(), 0);
    assert_eq!(
        client.list_by_owner(&collector).get(0).unwrap().id,
        badge_id
    );

    // The milestone stays claimed by the original recipient after a transfer.
    let badge = client.get_badge(&badge_id);
    assert_eq!(badge.minted_to, hunter);
    assert!(client.has_claimed(&hunter, &BadgeKind::MissionComplete, &5));
    assert_eq!(
        client.try_mint_badge(&admin, &hunter, &mission_badge(&env, 5, false)),
        Err(Ok(BadgeError::AlreadyMinted))
    );
}

#[test]
fn test_transfer_of_soulbound_badge_rejected() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);
    let collector = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 5, true));

    assert_eq!(
        client.try_transfer(&hunter, &collector, &badge_id),
        Err(Ok(BadgeError::SoulboundBadge))
    );
    assert_eq!(client.owner_of(&badge_id), hunter);
}

#[test]
fn test_transfer_by_non_owner_rejected() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);
    let stranger = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 5, false));

    assert_eq!(
        client.try_transfer(&stranger, &stranger, &badge_id),
        Err(Ok(BadgeError::NotBadgeOwner))
    );
}

#[test]
fn test_self_transfer_rejected() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 5, false));

    assert_eq!(
        client.try_transfer(&hunter, &hunter, &badge_id),
        Err(Ok(BadgeError::SelfTransfer))
    );
    assert_eq!(client.balance_of(&hunter), 1);
}

#[test]
fn test_owner_can_burn_and_badge_can_be_reissued() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 8, true));

    client.burn(&hunter, &badge_id);

    assert!(!client.badge_exists(&badge_id));
    assert_eq!(client.balance_of(&hunter), 0);
    assert!(!client.has_claimed(&hunter, &BadgeKind::MissionComplete, &8));

    let reissued = client.mint_badge(&admin, &hunter, &mission_badge(&env, 8, true));
    assert_eq!(reissued, 2);
    assert_eq!(client.total_minted(), 2);
}

#[test]
fn test_admin_can_burn_any_badge() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 8, true));

    client.burn(&admin, &badge_id);

    assert!(!client.badge_exists(&badge_id));
}

#[test]
fn test_burn_by_stranger_rejected() {
    let (env, client, admin) = setup();
    let hunter = Address::generate(&env);
    let stranger = Address::generate(&env);

    let badge_id = client.mint_badge(&admin, &hunter, &mission_badge(&env, 8, true));

    assert_eq!(
        client.try_burn(&stranger, &badge_id),
        Err(Ok(BadgeError::NotAuthorized))
    );
    assert!(client.badge_exists(&badge_id));
}
