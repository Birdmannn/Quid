extern crate std;

use super::*;
use quid_store::{QuidStoreContract, QuidStoreContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
};

fn config(env: &Env, store: &Address, token: &Address, name: &str) -> TemplateConfig {
    TemplateConfig {
        store: store.clone(),
        name: String::from_str(env, name),
        title: String::from_str(env, "Starter feedback mission"),
        description_cid: String::from_str(env, "QmSafeDefaults"),
        reward: Reward {
            reward_token: token.clone(),
            reward_amount: 100,
        },
        max_participants: 5,
        min_asset: MinAsset {
            min_asset_token: None,
            min_asset_amount: 0,
        },
    }
}

#[test]
fn template_create_works_with_store() {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(42);

    let admin = Address::generate(&env);
    let owner = Address::generate(&env);
    let token_admin = Address::generate(&env);
    let token = env
        .register_stellar_asset_contract_v2(token_admin.clone())
        .address();
    StellarAssetClient::new(&env, &token).mint(&owner, &1_000);

    let store_id = env.register(QuidStoreContract, ());
    let factory_id = env.register(QuidMissionFactoryContract, ());
    let factory = QuidMissionFactoryContractClient::new(&env, &factory_id);
    factory.initialize(&admin);

    let template_id = factory.register_template(&config(&env, &store_id, &token, "Starter"));
    let mission_id = factory.create_from_template(&template_id, &owner);

    let mission = QuidStoreContractClient::new(&env, &store_id).get_mission(&mission_id);
    assert_eq!(mission.owner, owner);
    assert_eq!(
        mission.title,
        String::from_str(&env, "Starter feedback mission")
    );
    assert_eq!(mission.reward_amount, 100);
    assert_eq!(mission.max_participants, 5);
    assert_eq!(TokenClient::new(&env, &token).balance(&store_id), 500);
}

#[test]
fn registers_and_lists_templates_in_order() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let store = Address::generate(&env);
    let token = Address::generate(&env);
    let factory_id = env.register(QuidMissionFactoryContract, ());
    let factory = QuidMissionFactoryContractClient::new(&env, &factory_id);
    factory.initialize(&admin);

    assert_eq!(
        factory.register_template(&config(&env, &store, &token, "One")),
        1
    );
    assert_eq!(
        factory.register_template(&config(&env, &store, &token, "Two")),
        2
    );
    let templates = factory.list_templates();
    assert_eq!(templates.len(), 2);
    assert_eq!(
        templates.get(0).unwrap().name,
        String::from_str(&env, "One")
    );
    assert_eq!(
        templates.get(1).unwrap().name,
        String::from_str(&env, "Two")
    );
}

#[test]
fn rejects_unsafe_defaults() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let factory_id = env.register(QuidMissionFactoryContract, ());
    let factory = QuidMissionFactoryContractClient::new(&env, &factory_id);
    factory.initialize(&admin);

    let mut invalid = config(
        &env,
        &Address::generate(&env),
        &Address::generate(&env),
        "Unsafe",
    );
    invalid.max_participants = 0;
    assert_eq!(
        factory.try_register_template(&invalid),
        Err(Ok(FactoryError::InvalidTemplate))
    );
}

#[test]
fn missing_template_is_reported_before_store_call() {
    let env = Env::default();
    env.mock_all_auths();
    let owner = Address::generate(&env);
    let factory_id = env.register(QuidMissionFactoryContract, ());
    let factory = QuidMissionFactoryContractClient::new(&env, &factory_id);

    assert_eq!(
        factory.try_create_from_template(&99, &owner),
        Err(Ok(FactoryError::TemplateNotFound))
    );
}
