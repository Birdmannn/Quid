#![cfg(test)]

use crate::{QuidLeaderboardContract, QuidLeaderboardContractClient};
use soroban_sdk::{testutils::Address as _, testutils::Ledger as _, Address, Env};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Deploy the contract, initialize it, and return (env, contract_id, admin, recorder).
fn setup() -> (Env, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidLeaderboardContract, ());
    let admin = Address::generate(&env);
    let recorder = Address::generate(&env);

    let client = QuidLeaderboardContractClient::new(&env, &contract_id);
    client.initialize(&admin, &recorder);

    (env, contract_id, admin, recorder)
}

fn client<'a>(env: &'a Env, contract_id: &'a Address) -> QuidLeaderboardContractClient<'a> {
    QuidLeaderboardContractClient::new(env, contract_id)
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_stores_admin_and_recorder() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);

    assert_eq!(c.get_admin(), admin);
    assert_eq!(c.get_recorder(), recorder);
    assert_eq!(c.get_epoch_count(), 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_double_initialize_panics() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);
    // Second call must fail with AlreadyInitialized = 1.
    c.initialize(&admin, &recorder);
}

// ---------------------------------------------------------------------------
// Authorization
// ---------------------------------------------------------------------------

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_start_epoch() {
    let (env, contract_id, _admin, _recorder) = setup();
    let c = client(&env, &contract_id);
    let stranger = Address::generate(&env);
    c.start_epoch(&stranger);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_recorder_cannot_record_score() {
    let (env, contract_id, admin, _recorder) = setup();
    let c = client(&env, &contract_id);
    c.start_epoch(&admin);

    let stranger = Address::generate(&env);
    let hunter = Address::generate(&env);
    c.record_score(&stranger, &hunter, &10_i64);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_end_epoch() {
    let (env, contract_id, admin, _recorder) = setup();
    let c = client(&env, &contract_id);
    c.start_epoch(&admin);

    let stranger = Address::generate(&env);
    c.end_epoch(&stranger);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_non_admin_cannot_set_recorder() {
    let (env, contract_id, _admin, _recorder) = setup();
    let c = client(&env, &contract_id);
    let stranger = Address::generate(&env);
    let new_recorder = Address::generate(&env);
    c.set_recorder(&stranger, &new_recorder);
}

// ---------------------------------------------------------------------------
// Epoch lifecycle
// ---------------------------------------------------------------------------

#[test]
fn test_start_epoch_increments_count_and_returns_id() {
    let (env, contract_id, admin, _recorder) = setup();
    let c = client(&env, &contract_id);

    let epoch_id = c.start_epoch(&admin);
    assert_eq!(epoch_id, 1);
    assert_eq!(c.get_epoch_count(), 1);

    let epoch = c.get_epoch(&epoch_id);
    assert_eq!(epoch.id, 1);
    assert_eq!(epoch.ended_at, 0); // still active
    assert_eq!(epoch.entry_count, 0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_start_epoch_while_active_panics() {
    let (env, contract_id, admin, _recorder) = setup();
    let c = client(&env, &contract_id);
    c.start_epoch(&admin);
    // Second start must fail with EpochAlreadyActive = 4.
    c.start_epoch(&admin);
}

#[test]
fn test_end_epoch_clears_active_and_stamps_timestamp() {
    let (env, contract_id, admin, _recorder) = setup();
    let c = client(&env, &contract_id);

    let epoch_id = c.start_epoch(&admin);

    // Advance the ledger timestamp so ended_at is distinguishable from 0.
    env.ledger().with_mut(|li| li.timestamp = 1_000);

    let closed_id = c.end_epoch(&admin);
    assert_eq!(closed_id, epoch_id);

    let epoch = c.get_epoch(&epoch_id);
    assert_eq!(epoch.ended_at, 1_000);

    // After ending, there is no active epoch — a new one can be started.
    let id2 = c.start_epoch(&admin);
    assert_eq!(id2, epoch_id + 1);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_end_epoch_without_active_panics() {
    let (env, contract_id, admin, _recorder) = setup();
    let c = client(&env, &contract_id);
    // No epoch started; must fail with NoActiveEpoch = 3.
    c.end_epoch(&admin);
}

#[test]
fn test_multiple_epochs_sequential() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);
    let hunter = Address::generate(&env);

    // Epoch 1
    let id1 = c.start_epoch(&admin);
    c.record_score(&recorder, &hunter, &50_i64);
    c.end_epoch(&admin);

    // Epoch 2
    let id2 = c.start_epoch(&admin);
    c.record_score(&recorder, &hunter, &80_i64);
    c.end_epoch(&admin);

    assert_eq!(id1, 1);
    assert_eq!(id2, 2);
    assert_eq!(c.get_epoch_count(), 2);

    // Scores are isolated per epoch.
    assert_eq!(c.get_score(&id1, &hunter), 50);
    assert_eq!(c.get_score(&id2, &hunter), 80);
}

// ---------------------------------------------------------------------------
// record_score
// ---------------------------------------------------------------------------

#[test]
fn test_record_score_accumulates() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);
    let hunter = Address::generate(&env);

    let epoch_id = c.start_epoch(&admin);

    let s1 = c.record_score(&recorder, &hunter, &30_i64);
    assert_eq!(s1, 30);

    let s2 = c.record_score(&recorder, &hunter, &20_i64);
    assert_eq!(s2, 50);

    // Negative delta (penalty).
    let s3 = c.record_score(&recorder, &hunter, &-10_i64);
    assert_eq!(s3, 40);

    assert_eq!(c.get_score(&epoch_id, &hunter), 40);
}

#[test]
fn test_record_score_tracks_entry_count() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);

    let epoch_id = c.start_epoch(&admin);

    let h1 = Address::generate(&env);
    let h2 = Address::generate(&env);

    c.record_score(&recorder, &h1, &10_i64);
    assert_eq!(c.get_epoch(&epoch_id).entry_count, 1);

    c.record_score(&recorder, &h2, &20_i64);
    assert_eq!(c.get_epoch(&epoch_id).entry_count, 2);

    // Recording again for h1 must not increase entry_count.
    c.record_score(&recorder, &h1, &5_i64);
    assert_eq!(c.get_epoch(&epoch_id).entry_count, 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_record_score_no_active_epoch_panics() {
    let (env, contract_id, _admin, recorder) = setup();
    let c = client(&env, &contract_id);
    let hunter = Address::generate(&env);
    // No epoch started yet.
    c.record_score(&recorder, &hunter, &10_i64);
}

// ---------------------------------------------------------------------------
// get_top_n
// ---------------------------------------------------------------------------

#[test]
fn test_get_top_n_returns_correct_ranking() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);

    let epoch_id = c.start_epoch(&admin);

    let h1 = Address::generate(&env);
    let h2 = Address::generate(&env);
    let h3 = Address::generate(&env);

    c.record_score(&recorder, &h1, &10_i64);
    c.record_score(&recorder, &h2, &50_i64);
    c.record_score(&recorder, &h3, &30_i64);

    let top3 = c.get_top_n(&epoch_id, &3);
    assert_eq!(top3.len(), 3);
    assert_eq!(top3.get(0).unwrap().score, 50);
    assert_eq!(top3.get(0).unwrap().rank, 1);
    assert_eq!(top3.get(1).unwrap().score, 30);
    assert_eq!(top3.get(1).unwrap().rank, 2);
    assert_eq!(top3.get(2).unwrap().score, 10);
    assert_eq!(top3.get(2).unwrap().rank, 3);

    let top1 = c.get_top_n(&epoch_id, &1);
    assert_eq!(top1.len(), 1);
    assert_eq!(top1.get(0).unwrap().hunter, h2);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_get_top_n_zero_panics() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);
    let epoch_id = c.start_epoch(&admin);
    let h = Address::generate(&env);
    c.record_score(&recorder, &h, &10_i64);
    c.get_top_n(&epoch_id, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_get_top_n_exceeds_entry_count_panics() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);
    let epoch_id = c.start_epoch(&admin);
    let h = Address::generate(&env);
    c.record_score(&recorder, &h, &10_i64); // 1 entry
    c.get_top_n(&epoch_id, &2); // asking for 2
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_get_top_n_unknown_epoch_panics() {
    let (env, contract_id, _admin, _recorder) = setup();
    let c = client(&env, &contract_id);
    c.get_top_n(&999, &1);
}

#[test]
fn test_get_top_n_on_ended_epoch() {
    let (env, contract_id, admin, recorder) = setup();
    let c = client(&env, &contract_id);

    let epoch_id = c.start_epoch(&admin);
    let h1 = Address::generate(&env);
    let h2 = Address::generate(&env);
    c.record_score(&recorder, &h1, &100_i64);
    c.record_score(&recorder, &h2, &200_i64);
    c.end_epoch(&admin);

    // Query on a closed epoch must still work.
    let top2 = c.get_top_n(&epoch_id, &2);
    assert_eq!(top2.get(0).unwrap().score, 200);
    assert_eq!(top2.get(1).unwrap().score, 100);
}

// ---------------------------------------------------------------------------
// set_recorder
// ---------------------------------------------------------------------------

#[test]
fn test_admin_can_rotate_recorder() {
    let (env, contract_id, admin, _recorder) = setup();
    let c = client(&env, &contract_id);

    let new_recorder = Address::generate(&env);
    c.set_recorder(&admin, &new_recorder);
    assert_eq!(c.get_recorder(), new_recorder);

    // Old recorder is no longer authorized.
    let epoch_id = c.start_epoch(&admin);
    let hunter = Address::generate(&env);

    // New recorder works.
    let score = c.record_score(&new_recorder, &hunter, &42_i64);
    assert_eq!(score, 42);
    let _ = epoch_id;
}
