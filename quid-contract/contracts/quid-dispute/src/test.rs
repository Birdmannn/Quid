#![cfg(test)]

use super::*;
use crate::types::{CreateDisputeParams, DisputeStatus};
use soroban_sdk::{
    testutils::{Address as _, Events as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

fn setup_test_env() -> (Env, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|li| {
        li.timestamp = 1_000;
    });

    let contract_id = env.register(QuidDisputeContract, ());
    let token_admin = Address::generate(&env);
    let token_contract = env.register_stellar_asset_contract_v2(token_admin.clone());
    let token_address = token_contract.address();

    (env, contract_id, token_address)
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

fn params(
    env: &Env,
    respondent: &Address,
    arbiter: Option<Address>,
    token: &Address,
    bond: i128,
    timeout_secs: u64,
) -> CreateDisputeParams {
    CreateDisputeParams {
        mission_id: 1,
        respondent: respondent.clone(),
        arbiter,
        bond_token: token.clone(),
        bond_amount: bond,
        reason_cid: String::from_str(env, "QmDisputeReason"),
        timeout_secs,
    }
}

fn open_dispute(
    env: &Env,
    client: &QuidDisputeContractClient,
    token: &Address,
    hunter_bond: i128,
    timeout_secs: u64,
    with_arbiter: bool,
) -> (Address, Address, Option<Address>, u64) {
    let hunter = Address::generate(env);
    let respondent = Address::generate(env);
    let arbiter = if with_arbiter {
        Some(Address::generate(env))
    } else {
        None
    };
    mint(env, token, &hunter, 10_000);
    mint(env, token, &respondent, 10_000);

    let id = client.create_dispute(
        &hunter,
        &params(
            env,
            &respondent,
            arbiter.clone(),
            token,
            hunter_bond,
            timeout_secs,
        ),
    );
    (hunter, respondent, arbiter, id)
}

#[test]
fn test_create_dispute_stakes_hunter_bond() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);
    let hunter_bond = 100_i128;

    let (hunter, respondent, arbiter, dispute_id) =
        open_dispute(&env, &client, &token, hunter_bond, 86_400, true);

    assert_eq!(dispute_id, 1);
    assert_eq!(client.get_dispute_count(), 1);
    assert_eq!(token_client.balance(&contract_id), hunter_bond);
    assert_eq!(client.get_open_dispute(&1, &hunter), Some(dispute_id));

    let dispute = client.get_dispute(&dispute_id);
    assert_eq!(dispute.hunter, hunter);
    assert_eq!(dispute.respondent, respondent);
    assert_eq!(dispute.arbiter, arbiter);
    assert_eq!(dispute.hunter_bond, hunter_bond);
    assert_eq!(dispute.respondent_bond, 0);
    assert_eq!(dispute.status, DisputeStatus::Open);
    assert_eq!(dispute.created_at, 1_000);
    assert_eq!(dispute.deadline, 1_000 + 86_400);
}

#[test]
fn test_stake_bond_respondent_counter() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);
    let hunter_bond = 100_i128;
    let respondent_bond = 150_i128;

    let (_hunter, respondent, _, dispute_id) =
        open_dispute(&env, &client, &token, hunter_bond, 86_400, true);

    client.stake_bond(&dispute_id, &respondent, &respondent_bond);

    let dispute = client.get_dispute(&dispute_id);
    assert_eq!(dispute.hunter_bond, hunter_bond);
    assert_eq!(dispute.respondent_bond, respondent_bond);
    assert_eq!(
        token_client.balance(&contract_id),
        hunter_bond + respondent_bond
    );
}

#[test]
fn test_stake_bond_hunter_can_add() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let extra = 25_i128;

    let (hunter, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, false);

    client.stake_bond(&dispute_id, &hunter, &extra);
    assert_eq!(client.get_dispute(&dispute_id).hunter_bond, 125);
}

#[test]
fn test_resolve_by_arbiter_hunter_wins_pot() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);
    let hunter_bond = 100_i128;
    let respondent_bond = 80_i128;

    let (hunter, respondent, arbiter, dispute_id) =
        open_dispute(&env, &client, &token, hunter_bond, 86_400, true);
    client.stake_bond(&dispute_id, &respondent, &respondent_bond);

    let hunter_before = token_client.balance(&hunter);
    client.resolve_by_arbiter(&dispute_id, &arbiter.unwrap(), &true);

    let dispute = client.get_dispute(&dispute_id);
    assert_eq!(dispute.status, DisputeStatus::ResolvedHunter);
    assert_eq!(dispute.hunter_bond, 0);
    assert_eq!(dispute.respondent_bond, 0);
    assert_eq!(
        token_client.balance(&hunter),
        hunter_before + hunter_bond + respondent_bond
    );
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(client.get_open_dispute(&1, &hunter), None);
}

#[test]
fn test_resolve_by_arbiter_respondent_wins_pot() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);
    let hunter_bond = 100_i128;
    let respondent_bond = 80_i128;

    let (_hunter, respondent, arbiter, dispute_id) =
        open_dispute(&env, &client, &token, hunter_bond, 86_400, true);
    client.stake_bond(&dispute_id, &respondent, &respondent_bond);

    let respondent_before = token_client.balance(&respondent);
    client.resolve_by_arbiter(&dispute_id, &arbiter.unwrap(), &false);

    let dispute = client.get_dispute(&dispute_id);
    assert_eq!(dispute.status, DisputeStatus::ResolvedRespondent);
    assert_eq!(
        token_client.balance(&respondent),
        respondent_before + hunter_bond + respondent_bond
    );
}

#[test]
fn test_timeout_release_refunds_both_bonds() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);
    let hunter_bond = 100_i128;
    let respondent_bond = 60_i128;

    let (hunter, respondent, _, dispute_id) =
        open_dispute(&env, &client, &token, hunter_bond, 86_400, false);
    client.stake_bond(&dispute_id, &respondent, &respondent_bond);

    let hunter_before = token_client.balance(&hunter);
    let respondent_before = token_client.balance(&respondent);

    env.ledger().with_mut(|li| {
        li.timestamp = 1_000 + 86_400;
    });
    client.timeout_release(&dispute_id);

    let dispute = client.get_dispute(&dispute_id);
    assert_eq!(dispute.status, DisputeStatus::TimedOut);
    assert_eq!(token_client.balance(&hunter), hunter_before + hunter_bond);
    assert_eq!(
        token_client.balance(&respondent),
        respondent_before + respondent_bond
    );
    assert_eq!(token_client.balance(&contract_id), 0);
    assert_eq!(client.get_open_dispute(&1, &hunter), None);
}

#[test]
fn test_timeout_release_hunter_only_bond() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);

    let (hunter, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 1, false);

    let hunter_before = token_client.balance(&hunter);
    env.ledger().with_mut(|li| {
        li.timestamp = 1_001;
    });
    client.timeout_release(&dispute_id);

    assert_eq!(token_client.balance(&hunter), hunter_before + 100);
    assert_eq!(
        client.get_dispute(&dispute_id).status,
        DisputeStatus::TimedOut
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_timeout_release_before_deadline() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (_, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, false);
    client.timeout_release(&dispute_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_resolve_after_deadline_fails() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (_, _, arbiter, dispute_id) = open_dispute(&env, &client, &token, 100, 10, true);

    env.ledger().with_mut(|li| {
        li.timestamp = 1_010;
    });
    client.resolve_by_arbiter(&dispute_id, &arbiter.unwrap(), &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #8)")]
fn test_resolve_without_arbiter_fails() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (_, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, false);
    let stranger = Address::generate(&env);
    client.resolve_by_arbiter(&dispute_id, &stranger, &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_resolve_wrong_arbiter_fails() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (_, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, true);
    let impostor = Address::generate(&env);
    client.resolve_by_arbiter(&dispute_id, &impostor, &true);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_stake_bond_unauthorized() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (_, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, false);
    let stranger = Address::generate(&env);
    mint(&env, &token, &stranger, 1_000);
    client.stake_bond(&dispute_id, &stranger, &10);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_prevent_duplicate_open_dispute() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (hunter, respondent, _, _) = open_dispute(&env, &client, &token, 100, 86_400, false);

    client.create_dispute(
        &hunter,
        &params(&env, &respondent, None, &token, 50, 86_400),
    );
}

#[test]
fn test_new_dispute_allowed_after_timeout() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (hunter, respondent, _, first_id) = open_dispute(&env, &client, &token, 100, 10, false);

    env.ledger().with_mut(|li| {
        li.timestamp = 1_010;
    });
    client.timeout_release(&first_id);

    let second_id = client.create_dispute(
        &hunter,
        &params(&env, &respondent, None, &token, 50, 86_400),
    );
    assert_eq!(second_id, 2);
    assert_eq!(client.get_open_dispute(&1, &hunter), Some(2));
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_create_dispute_zero_bond() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let hunter = Address::generate(&env);
    let respondent = Address::generate(&env);
    client.create_dispute(&hunter, &params(&env, &respondent, None, &token, 0, 86_400));
}

#[test]
#[should_panic(expected = "Error(Contract, #9)")]
fn test_create_dispute_zero_timeout() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let hunter = Address::generate(&env);
    let respondent = Address::generate(&env);
    mint(&env, &token, &hunter, 1_000);
    client.create_dispute(&hunter, &params(&env, &respondent, None, &token, 10, 0));
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_create_dispute_hunter_is_respondent() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let hunter = Address::generate(&env);
    mint(&env, &token, &hunter, 1_000);
    client.create_dispute(&hunter, &params(&env, &hunter, None, &token, 10, 86_400));
}

#[test]
#[should_panic(expected = "Error(Contract, #10)")]
fn test_create_dispute_arbiter_is_party() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let hunter = Address::generate(&env);
    let respondent = Address::generate(&env);
    mint(&env, &token, &hunter, 1_000);
    client.create_dispute(
        &hunter,
        &params(&env, &respondent, Some(hunter.clone()), &token, 10, 86_400),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_get_dispute_not_found() {
    let (env, contract_id, _token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let _ = client.get_dispute(&999);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_stake_bond_zero_amount() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (hunter, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, false);
    client.stake_bond(&dispute_id, &hunter, &0);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_cannot_resolve_twice() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (_, _, arbiter, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, true);
    let arbiter = arbiter.unwrap();
    client.resolve_by_arbiter(&dispute_id, &arbiter, &true);
    client.resolve_by_arbiter(&dispute_id, &arbiter, &false);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_cannot_stake_after_resolved() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (hunter, _, arbiter, dispute_id) = open_dispute(&env, &client, &token, 100, 86_400, true);
    client.resolve_by_arbiter(&dispute_id, &arbiter.unwrap(), &true);
    client.stake_bond(&dispute_id, &hunter, &10);
}

#[test]
#[should_panic(expected = "Error(Contract, #7)")]
fn test_cannot_stake_after_deadline() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let (hunter, _, _, dispute_id) = open_dispute(&env, &client, &token, 100, 10, false);
    env.ledger().with_mut(|li| {
        li.timestamp = 1_010;
    });
    client.stake_bond(&dispute_id, &hunter, &10);
}

#[test]
fn test_happy_path_create_stake_resolve() {
    let (env, contract_id, token) = setup_test_env();
    let client = QuidDisputeContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);

    let hunter = Address::generate(&env);
    let respondent = Address::generate(&env);
    let arbiter = Address::generate(&env);
    mint(&env, &token, &hunter, 1_000);
    mint(&env, &token, &respondent, 1_000);

    let dispute_id = client.create_dispute(
        &hunter,
        &params(&env, &respondent, Some(arbiter.clone()), &token, 40, 86_400),
    );
    client.stake_bond(&dispute_id, &respondent, &40);

    let hunter_before = token_client.balance(&hunter);
    client.resolve_by_arbiter(&dispute_id, &arbiter, &true);

    assert_eq!(
        client.get_dispute(&dispute_id).status,
        DisputeStatus::ResolvedHunter
    );
    assert_eq!(token_client.balance(&hunter), hunter_before + 80);
    assert_eq!(token_client.balance(&contract_id), 0);
}

#[test]
fn test_create_dispute_publishes_events() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidDisputeContract, ());
    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    let client = QuidDisputeContractClient::new(&env, &contract_id);

    let hunter = Address::generate(&env);
    let respondent = Address::generate(&env);
    mint(&env, &token_address, &hunter, 1_000);

    client.create_dispute(
        &hunter,
        &params(&env, &respondent, None, &token_address, 40, 86_400),
    );

    let events = env.events().all();
    assert!(events.len() >= 2);
    assert_eq!(events.last().unwrap().0, contract_id);
}
