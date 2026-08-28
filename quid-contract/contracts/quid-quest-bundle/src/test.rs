#![cfg(test)]

use super::*;
use crate::types::BundleStatus;
use soroban_sdk::{
    testutils::{Address as _, Events},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

const BONUS: i128 = 100;
const MAX_CLAIMS: u32 = 2;
const MINT: i128 = 10_000;

struct Fixture {
    env: Env,
    client: QuidQuestBundleContractClient<'static>,
    token: Address,
    token_client: TokenClient<'static>,
    owner: Address,
    hunter: Address,
    /// The address authorized to report completions (the `quid-store` stand-in).
    hook: Address,
}

fn setup() -> Fixture {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidQuestBundleContract, ());
    let client = QuidQuestBundleContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let hook = Address::generate(&env);
    client.initialize(&admin);
    client.set_payout_hook(&admin, &hook);

    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let owner = Address::generate(&env);
    StellarAssetClient::new(&env, &token).mint(&owner, &MINT);

    let token_client = TokenClient::new(&env, &token);
    let hunter = Address::generate(&env);

    Fixture {
        env,
        client,
        token,
        token_client,
        owner,
        hunter,
        hook,
    }
}

/// A funded bundle with `mission_count` missions already added.
fn bundle_with_missions(f: &Fixture, missions: &[u64]) -> u64 {
    let bundle_id = f.client.create_bundle(
        &f.owner,
        &String::from_str(&f.env, "Feedback campaign"),
        &String::from_str(&f.env, "QmCampaign"),
        &f.token,
        &BONUS,
        &MAX_CLAIMS,
    );
    for mission_id in missions {
        f.client.add_mission(&bundle_id, mission_id);
    }
    bundle_id
}

fn complete_all(f: &Fixture, bundle_id: u64, hunter: &Address, missions: &[u64]) {
    for mission_id in missions {
        f.client
            .record_completion(&f.hook, &bundle_id, hunter, mission_id);
    }
}

// ---------------------------------------------------------------------------
// Bootstrap
// ---------------------------------------------------------------------------

#[test]
fn initialize_sets_admin_and_rejects_a_second_call() {
    let env = Env::default();
    env.mock_all_auths();
    let client =
        QuidQuestBundleContractClient::new(&env, &env.register(QuidQuestBundleContract, ()));
    let admin = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(client.get_admin(), admin);
    assert_eq!(
        client.try_initialize(&admin),
        Err(Ok(QuestBundleError::AlreadyInitialized))
    );
}

#[test]
fn only_the_admin_can_set_the_payout_hook() {
    let f = setup();
    let stranger = Address::generate(&f.env);

    assert_eq!(
        f.client
            .try_set_payout_hook(&stranger, &Address::generate(&f.env)),
        Err(Ok(QuestBundleError::NotAuthorized))
    );
    assert_eq!(f.client.get_payout_hook(), f.hook);
}

// ---------------------------------------------------------------------------
// Campaign setup
// ---------------------------------------------------------------------------

#[test]
fn create_bundle_escrows_the_whole_bonus_pool() {
    let f = setup();

    let bundle_id = bundle_with_missions(&f, &[]);
    let bundle = f.client.get_bundle(&bundle_id);
    let escrowed = BONUS * MAX_CLAIMS as i128;

    assert_eq!(bundle.id, 1);
    assert_eq!(bundle.owner, f.owner);
    assert_eq!(bundle.escrow_balance, escrowed);
    assert_eq!(bundle.mission_count, 0);
    assert_eq!(bundle.status, BundleStatus::Active);
    assert_eq!(f.token_client.balance(&f.owner), MINT - escrowed);
    assert_eq!(
        f.token_client.balance(&f.client.address),
        escrowed,
        "the contract holds exactly what it may owe"
    );
    assert_eq!(f.client.get_bundle_count(), 1);
}

#[test]
fn create_bundle_rejects_a_negative_bonus_and_a_bonus_with_no_claim_slots() {
    let f = setup();
    let title = String::from_str(&f.env, "Campaign");
    let cid = String::from_str(&f.env, "QmCampaign");

    assert_eq!(
        f.client
            .try_create_bundle(&f.owner, &title, &cid, &f.token, &-1, &MAX_CLAIMS),
        Err(Ok(QuestBundleError::InvalidAmount))
    );
    assert_eq!(
        f.client
            .try_create_bundle(&f.owner, &title, &cid, &f.token, &BONUS, &0),
        Err(Ok(QuestBundleError::InvalidInput))
    );
    assert_eq!(f.token_client.balance(&f.owner), MINT, "nothing escrowed");
}

#[test]
fn add_mission_tracks_the_required_set_in_order() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[7, 9]);

    assert_eq!(f.client.get_bundle(&bundle_id).mission_count, 2);
    assert_eq!(f.client.get_mission_at(&bundle_id, &0), 7);
    assert_eq!(f.client.get_mission_at(&bundle_id, &1), 9);
    assert!(f.client.is_mission_in_bundle(&bundle_id, &7));
    assert!(!f.client.is_mission_in_bundle(&bundle_id, &8));
}

#[test]
fn the_same_mission_cannot_be_added_twice() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[7]);

    assert_eq!(
        f.client.try_add_mission(&bundle_id, &7),
        Err(Ok(QuestBundleError::MissionAlreadyAdded))
    );
    assert_eq!(f.client.get_bundle(&bundle_id).mission_count, 1);
}

// ---------------------------------------------------------------------------
// Completion reporting
// ---------------------------------------------------------------------------

#[test]
fn only_the_payout_hook_can_report_a_completion() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[7]);
    let stranger = Address::generate(&f.env);

    assert_eq!(
        f.client
            .try_record_completion(&stranger, &bundle_id, &f.hunter, &7),
        Err(Ok(QuestBundleError::NotAuthorized))
    );
    assert_eq!(f.client.get_completed_count(&bundle_id, &f.hunter), 0);
}

#[test]
fn a_completion_reported_twice_does_not_double_count() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[7, 9]);

    assert_eq!(
        f.client
            .record_completion(&f.hook, &bundle_id, &f.hunter, &7),
        1
    );
    assert_eq!(
        f.client
            .try_record_completion(&f.hook, &bundle_id, &f.hunter, &7),
        Err(Ok(QuestBundleError::CompletionAlreadyRecorded))
    );
    assert_eq!(f.client.get_completed_count(&bundle_id, &f.hunter), 1);
    assert!(f.client.is_completed(&bundle_id, &f.hunter, &7));
    assert!(!f.client.is_bundle_complete_for(&bundle_id, &f.hunter));
}

#[test]
fn a_mission_outside_the_bundle_cannot_be_completed() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[7]);

    assert_eq!(
        f.client
            .try_record_completion(&f.hook, &bundle_id, &f.hunter, &42),
        Err(Ok(QuestBundleError::MissionNotInBundle))
    );
}

// ---------------------------------------------------------------------------
// Completion claim
// ---------------------------------------------------------------------------

#[test]
fn finishing_every_mission_pays_the_bonus_once() {
    let f = setup();
    let missions = [7_u64, 9];
    let bundle_id = bundle_with_missions(&f, &missions);
    complete_all(&f, bundle_id, &f.hunter, &missions);

    assert!(f.client.is_bundle_complete_for(&bundle_id, &f.hunter));
    assert_eq!(
        f.client.claim_completion_bonus(&bundle_id, &f.hunter),
        BONUS
    );

    assert_eq!(f.token_client.balance(&f.hunter), BONUS);
    assert!(f.client.has_claimed(&bundle_id, &f.hunter));

    let bundle = f.client.get_bundle(&bundle_id);
    assert_eq!(bundle.claims_made, 1);
    assert_eq!(
        bundle.escrow_balance,
        BONUS * MAX_CLAIMS as i128 - BONUS,
        "escrow drops by exactly one bonus"
    );
    assert_eq!(
        f.token_client.balance(&f.client.address),
        bundle.escrow_balance,
        "the ledger agrees with the bundle's own accounting"
    );

    assert_eq!(
        f.client.try_claim_completion_bonus(&bundle_id, &f.hunter),
        Err(Ok(QuestBundleError::AlreadyClaimed))
    );
    assert_eq!(f.token_client.balance(&f.hunter), BONUS, "paid once");
}

#[test]
fn a_partial_run_cannot_claim() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[7, 9]);
    f.client
        .record_completion(&f.hook, &bundle_id, &f.hunter, &7);

    assert_eq!(
        f.client.try_claim_completion_bonus(&bundle_id, &f.hunter),
        Err(Ok(QuestBundleError::MissionsIncomplete))
    );
    assert_eq!(f.token_client.balance(&f.hunter), 0);
    assert!(!f.client.has_claimed(&bundle_id, &f.hunter));
}

#[test]
fn an_empty_bundle_has_nothing_to_claim() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[]);

    assert!(!f.client.is_bundle_complete_for(&bundle_id, &f.hunter));
    assert_eq!(
        f.client.try_claim_completion_bonus(&bundle_id, &f.hunter),
        Err(Ok(QuestBundleError::BundleEmpty))
    );
}

#[test]
fn the_bonus_slots_run_out_after_max_claims() {
    let f = setup();
    let missions = [7_u64];
    let bundle_id = bundle_with_missions(&f, &missions);

    let hunters = [
        f.hunter.clone(),
        Address::generate(&f.env),
        Address::generate(&f.env),
    ];
    for hunter in &hunters {
        complete_all(&f, bundle_id, hunter, &missions);
    }

    f.client.claim_completion_bonus(&bundle_id, &hunters[0]);
    f.client.claim_completion_bonus(&bundle_id, &hunters[1]);

    // MAX_CLAIMS bonuses were funded; the third hunter finished but cannot be
    // paid out of an empty escrow.
    assert_eq!(
        f.client.try_claim_completion_bonus(&bundle_id, &hunters[2]),
        Err(Ok(QuestBundleError::NoClaimsRemaining))
    );
    assert_eq!(f.client.get_bundle(&bundle_id).escrow_balance, 0);
    assert_eq!(f.token_client.balance(&f.client.address), 0);
}

#[test]
fn a_campaign_without_a_bonus_still_records_the_completion() {
    let f = setup();
    let bundle_id = f.client.create_bundle(
        &f.owner,
        &String::from_str(&f.env, "No bonus"),
        &String::from_str(&f.env, "QmCampaign"),
        &f.token,
        &0,
        &0,
    );
    f.client.add_mission(&bundle_id, &7);
    f.client
        .record_completion(&f.hook, &bundle_id, &f.hunter, &7);

    assert_eq!(f.client.claim_completion_bonus(&bundle_id, &f.hunter), 0);
    assert!(f.client.has_claimed(&bundle_id, &f.hunter));
    assert_eq!(f.token_client.balance(&f.hunter), 0);
    assert_eq!(
        f.client.try_claim_completion_bonus(&bundle_id, &f.hunter),
        Err(Ok(QuestBundleError::AlreadyClaimed))
    );
}

#[test]
fn the_required_set_is_frozen_once_a_hunter_has_claimed() {
    let f = setup();
    let missions = [7_u64];
    let bundle_id = bundle_with_missions(&f, &missions);
    complete_all(&f, bundle_id, &f.hunter, &missions);
    f.client.claim_completion_bonus(&bundle_id, &f.hunter);

    assert_eq!(
        f.client.try_add_mission(&bundle_id, &9),
        Err(Ok(QuestBundleError::InvalidState))
    );
    assert_eq!(f.client.get_bundle(&bundle_id).mission_count, 1);
}

// ---------------------------------------------------------------------------
// Cancellation
// ---------------------------------------------------------------------------

#[test]
fn cancelling_refunds_the_whole_untouched_escrow() {
    let f = setup();
    let bundle_id = bundle_with_missions(&f, &[7]);
    let escrowed = BONUS * MAX_CLAIMS as i128;

    assert_eq!(f.client.cancel_bundle(&bundle_id), escrowed);

    assert_eq!(f.token_client.balance(&f.owner), MINT);
    assert_eq!(f.token_client.balance(&f.client.address), 0);

    let bundle = f.client.get_bundle(&bundle_id);
    assert_eq!(bundle.status, BundleStatus::Cancelled);
    assert_eq!(bundle.escrow_balance, 0);
}

#[test]
fn cancelling_refunds_only_the_unclaimed_remainder() {
    let f = setup();
    let missions = [7_u64];
    let bundle_id = bundle_with_missions(&f, &missions);
    complete_all(&f, bundle_id, &f.hunter, &missions);
    f.client.claim_completion_bonus(&bundle_id, &f.hunter);

    let escrowed = BONUS * MAX_CLAIMS as i128;
    assert_eq!(f.client.cancel_bundle(&bundle_id), escrowed - BONUS);

    assert_eq!(f.token_client.balance(&f.hunter), BONUS, "claim stands");
    assert_eq!(f.token_client.balance(&f.owner), MINT - BONUS);
    assert_eq!(f.token_client.balance(&f.client.address), 0);
}

#[test]
fn a_cancelled_bundle_accepts_no_further_claims_completions_or_cancels() {
    let f = setup();
    let missions = [7_u64];
    let bundle_id = bundle_with_missions(&f, &missions);
    complete_all(&f, bundle_id, &f.hunter, &missions);
    f.client.cancel_bundle(&bundle_id);

    assert_eq!(
        f.client.try_claim_completion_bonus(&bundle_id, &f.hunter),
        Err(Ok(QuestBundleError::InvalidState))
    );
    assert_eq!(
        f.client.try_cancel_bundle(&bundle_id),
        Err(Ok(QuestBundleError::InvalidState))
    );
    assert_eq!(
        f.client
            .try_record_completion(&f.hook, &bundle_id, &Address::generate(&f.env), &7),
        Err(Ok(QuestBundleError::InvalidState))
    );
    assert_eq!(f.token_client.balance(&f.client.address), 0);
}

#[test]
fn an_unknown_bundle_is_a_typed_error() {
    let f = setup();

    assert_eq!(
        f.client.try_get_bundle(&404),
        Err(Ok(QuestBundleError::BundleNotFound))
    );
    assert_eq!(
        f.client.try_add_mission(&404, &7),
        Err(Ok(QuestBundleError::BundleNotFound))
    );
    assert!(!f.client.is_bundle_complete_for(&404, &f.hunter));
}

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

#[test]
fn the_claim_publishes_one_bonus_event() {
    let f = setup();
    let missions = [7_u64];
    let bundle_id = bundle_with_missions(&f, &missions);
    complete_all(&f, bundle_id, &f.hunter, &missions);

    let before = f.env.events().all().len();
    f.client.claim_completion_bonus(&bundle_id, &f.hunter);
    let published = f.env.events().all().len() - before;

    assert!(
        published >= 1,
        "the claim publishes at least its own BonusClaimedEvent"
    );
}
