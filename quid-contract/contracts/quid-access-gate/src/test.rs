use super::*;
use soroban_sdk::{testutils::Address as _, token, Address, Env};

fn create_token_contract<'a>(
    env: &Env,
    admin: &Address,
) -> (token::StellarAssetClient<'a>, token::TokenClient<'a>) {
    let token_address = env.register_stellar_asset_contract_v2(admin.clone());
    (
        token::StellarAssetClient::new(env, &token_address.address()),
        token::TokenClient::new(env, &token_address.address()),
    )
}

#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
}

#[test]
fn test_configure_token_balance_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    client.initialize(&admin);

    let min_amount = 1000i128;
    let rule_id = client.configure_rule(&GateType::TokenBalance, &token_address, &min_amount);

    assert_eq!(rule_id, 1);

    let rule = client.get_rule(&rule_id);
    assert_eq!(rule.id, rule_id);
    assert_eq!(rule.gate_type, GateType::TokenBalance);
    assert_eq!(rule.token_address, token_address);
    assert_eq!(rule.min_amount, min_amount);
    assert!(rule.active);
}

#[test]
fn test_configure_nft_ownership_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let nft_address = Address::generate(&env);

    client.initialize(&admin);

    let min_nfts = 1i128;
    let rule_id = client.configure_rule(&GateType::NftOwnership, &nft_address, &min_nfts);

    assert_eq!(rule_id, 1);

    let rule = client.get_rule(&rule_id);
    assert_eq!(rule.gate_type, GateType::NftOwnership);
    assert_eq!(rule.min_amount, min_nfts);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_invalid_amount_zero() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    client.initialize(&admin);
    client.configure_rule(&GateType::TokenBalance, &token_address, &0i128);
}

#[test]
#[should_panic(expected = "Error(Contract, #4)")]
fn test_invalid_amount_negative() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    client.initialize(&admin);
    client.configure_rule(&GateType::TokenBalance, &token_address, &-100i128);
}

#[test]
fn test_check_token_balance_access_pass() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token_admin, token_client) = create_token_contract(&env, &admin);
    token_admin.mint(&user, &5000);

    client.initialize(&admin);
    let rule_id = client.configure_rule(&GateType::TokenBalance, &token_client.address, &1000i128);

    let has_access = client.check_access(&user, &rule_id);
    assert!(has_access);
}

#[test]
fn test_check_token_balance_access_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token_admin, token_client) = create_token_contract(&env, &admin);
    token_admin.mint(&user, &500);

    client.initialize(&admin);
    let rule_id = client.configure_rule(&GateType::TokenBalance, &token_client.address, &1000i128);

    let has_access = client.check_access(&user, &rule_id);
    assert!(!has_access);
}

#[test]
fn test_check_nft_ownership_access_pass() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (nft_admin, nft_client) = create_token_contract(&env, &admin);
    nft_admin.mint(&user, &3);

    client.initialize(&admin);
    let rule_id = client.configure_rule(&GateType::NftOwnership, &nft_client.address, &2i128);

    let has_access = client.check_access(&user, &rule_id);
    assert!(has_access);
}

#[test]
fn test_check_nft_ownership_access_fail() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);
    let user_without_nft = Address::generate(&env);

    let (nft_admin, nft_client) = create_token_contract(&env, &admin);
    nft_admin.mint(&user, &1);

    client.initialize(&admin);
    let rule_id = client.configure_rule(&GateType::NftOwnership, &nft_client.address, &1i128);

    let has_access = client.check_access(&user_without_nft, &rule_id);
    assert!(!has_access);
}

#[test]
fn test_deactivate_rule() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    client.initialize(&admin);
    let rule_id = client.configure_rule(&GateType::TokenBalance, &token_address, &1000i128);

    client.deactivate_rule(&rule_id);

    let rule = client.get_rule(&rule_id);
    assert!(!rule.active);

    let user = Address::generate(&env);
    let has_access = client.check_access(&user, &rule_id);
    assert!(!has_access);
}

#[test]
fn test_check_multiple_access_all_pass() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token1_admin, token1_client) = create_token_contract(&env, &admin);
    let (token2_admin, token2_client) = create_token_contract(&env, &admin);

    token1_admin.mint(&user, &2000);
    token2_admin.mint(&user, &3000);

    client.initialize(&admin);
    let rule_id1 =
        client.configure_rule(&GateType::TokenBalance, &token1_client.address, &1000i128);
    let rule_id2 =
        client.configure_rule(&GateType::TokenBalance, &token2_client.address, &1500i128);

    let rule_ids = soroban_sdk::vec![&env, rule_id1, rule_id2];
    let has_access = client.check_multiple_access(&user, &rule_ids);
    assert!(has_access);
}

#[test]
fn test_check_multiple_access_one_fails() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    let (token1_admin, token1_client) = create_token_contract(&env, &admin);
    let (token2_admin, token2_client) = create_token_contract(&env, &admin);

    token1_admin.mint(&user, &2000);
    token2_admin.mint(&user, &500);

    client.initialize(&admin);
    let rule_id1 =
        client.configure_rule(&GateType::TokenBalance, &token1_client.address, &1000i128);
    let rule_id2 =
        client.configure_rule(&GateType::TokenBalance, &token2_client.address, &1500i128);

    let rule_ids = soroban_sdk::vec![&env, rule_id1, rule_id2];
    let has_access = client.check_multiple_access(&user, &rule_ids);
    assert!(!has_access);
}

#[test]
fn test_get_rule_count() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let token_address = Address::generate(&env);

    client.initialize(&admin);

    assert_eq!(client.get_rule_count(), 0);

    client.configure_rule(&GateType::TokenBalance, &token_address, &1000i128);
    assert_eq!(client.get_rule_count(), 1);

    client.configure_rule(&GateType::NftOwnership, &token_address, &1i128);
    assert_eq!(client.get_rule_count(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_rule_not_found_get() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    client.get_rule(&999);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_rule_not_found_check() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidAccessGateContract, ());
    let client = QuidAccessGateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin);
    client.check_access(&user, &999);
}
