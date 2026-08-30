use super::*;
use soroban_sdk::token::{Client as TokenClient, StellarAssetClient};
use soroban_sdk::{testutils::Address as _, Address, Env};

fn setup() -> (Env, Address, Address, Address, Address, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(QuidStakingContract, ());
    let client = QuidStakingContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let treasury = Address::generate(&env);
    let hunter = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();
    StellarAssetClient::new(&env, &token).mint(&hunter, &1_000);
    client.initialize(&admin, &treasury);
    (env, contract_id, admin, treasury, hunter, token)
}

#[test]
fn deposit_and_withdraw_require_available_stake() {
    let (env, contract_id, _admin, _treasury, hunter, token) = setup();
    let client = QuidStakingContractClient::new(&env, &contract_id);
    let token_client = TokenClient::new(&env, &token);
    client.deposit(&hunter, &token, &100);
    client.withdraw(&hunter, &token, &40);
    assert_eq!(client.get_balance(&hunter, &token), 60);
    assert_eq!(token_client.balance(&hunter), 940);
}

#[test]
fn locks_prevent_double_spend_and_can_be_released() {
    let (env, contract_id, admin, _treasury, hunter, token) = setup();
    let client = QuidStakingContractClient::new(&env, &contract_id);
    let locker = Address::generate(&env);
    client.set_locker(&admin, &locker, &true);
    client.deposit(&hunter, &token, &100);
    client.lock_for_mission(&locker, &1, &hunter, &token, &70);
    assert_eq!(client.get_locked_balance(&hunter, &token), 70);
    assert!(client
        .try_lock_for_mission(&locker, &2, &hunter, &token, &40)
        .is_err());
    client.unlock_for_mission(&locker, &1, &hunter, &token);
    client.lock_for_mission(&locker, &2, &hunter, &token, &100);
}

#[test]
fn slash_transfers_locked_stake_to_treasury() {
    let (env, contract_id, admin, treasury, hunter, token) = setup();
    let client = QuidStakingContractClient::new(&env, &contract_id);
    let locker = Address::generate(&env);
    let token_client = TokenClient::new(&env, &token);
    client.set_locker(&admin, &locker, &true);
    client.deposit(&hunter, &token, &100);
    client.lock_for_mission(&locker, &9, &hunter, &token, &60);
    client.slash_for_mission(&locker, &9, &hunter, &token);
    assert_eq!(token_client.balance(&treasury), 60);
    assert_eq!(client.get_balance(&hunter, &token), 40);
    assert_eq!(client.get_locked_balance(&hunter, &token), 0);
}
