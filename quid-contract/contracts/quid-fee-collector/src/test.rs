#![cfg(test)]

use super::*;
use soroban_sdk::{
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env,
};

const INITIAL_FEE_BPS: u32 = 250; // 2.5%

fn setup() -> (
    Env,
    QuidFeeCollectorContractClient<'static>,
    Address,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(QuidFeeCollectorContract, ());
    let client = QuidFeeCollectorContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin, &INITIAL_FEE_BPS);

    let token_admin = Address::generate(&env);
    let token_address = env
        .register_stellar_asset_contract_v2(token_admin)
        .address();

    (env, client, admin, token_address)
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

// -----------------------------------------------------------------------------
// Bootstrap
// -----------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin_and_rate() {
    let (_, client, admin, _) = setup();

    assert_eq!(client.get_admin(), admin);
    assert_eq!(client.get_fee_bps(), INITIAL_FEE_BPS);
}

#[test]
fn test_initialize_twice_fails() {
    let (env, client, _, _) = setup();
    let other = Address::generate(&env);

    assert_eq!(
        client.try_initialize(&other, &100),
        Err(Ok(FeeError::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_rejects_rate_above_one_hundred_percent() {
    let env = Env::default();
    env.mock_all_auths();

    let client =
        QuidFeeCollectorContractClient::new(&env, &env.register(QuidFeeCollectorContract, ()));
    let admin = Address::generate(&env);

    assert_eq!(
        client.try_initialize(&admin, &(MAX_FEE_BPS + 1)),
        Err(Ok(FeeError::InvalidFeeBps))
    );
}

#[test]
fn test_set_admin_transfers_control() {
    let (env, client, admin, _) = setup();
    let new_admin = Address::generate(&env);

    client.set_admin(&admin, &new_admin);

    assert_eq!(client.get_admin(), new_admin);
    // The old admin no longer passes the admin gate.
    assert_eq!(
        client.try_set_fee_bps(&admin, &10),
        Err(Ok(FeeError::NotAuthorized))
    );
}

// -----------------------------------------------------------------------------
// Fee rate + fee math
// -----------------------------------------------------------------------------

#[test]
fn test_admin_can_update_fee_bps() {
    let (_, client, admin, _) = setup();

    client.set_fee_bps(&admin, &500);

    assert_eq!(client.get_fee_bps(), 500);
}

#[test]
fn test_non_admin_cannot_update_fee_bps() {
    let (env, client, _, _) = setup();
    let stranger = Address::generate(&env);

    assert_eq!(
        client.try_set_fee_bps(&stranger, &0),
        Err(Ok(FeeError::NotAuthorized))
    );
    assert_eq!(client.get_fee_bps(), INITIAL_FEE_BPS);
}

#[test]
fn test_set_fee_bps_rejects_rate_above_one_hundred_percent() {
    let (_, client, admin, _) = setup();

    assert_eq!(
        client.try_set_fee_bps(&admin, &(MAX_FEE_BPS + 1)),
        Err(Ok(FeeError::InvalidFeeBps))
    );
    assert_eq!(client.get_fee_bps(), INITIAL_FEE_BPS);
}

#[test]
fn test_compute_fee_math() {
    let (_, client, admin, _) = setup();

    // 2.5% of 10_000 == 250
    assert_eq!(client.compute_fee(&10_000), 250);
    assert_eq!(client.compute_fee(&0), 0);
    // Rounds down: 2.5% of 39 == 0.975
    assert_eq!(client.compute_fee(&39), 0);
    // Rounds down: 2.5% of 199 == 4.975
    assert_eq!(client.compute_fee(&199), 4);

    client.set_fee_bps(&admin, &MAX_FEE_BPS);
    assert_eq!(client.compute_fee(&777), 777);

    client.set_fee_bps(&admin, &0);
    assert_eq!(client.compute_fee(&1_000_000), 0);
}

#[test]
fn test_compute_fee_rejects_negative_gross() {
    let (_, client, _, _) = setup();

    assert_eq!(
        client.try_compute_fee(&-1),
        Err(Ok(FeeError::InvalidAmount))
    );
}

#[test]
fn test_compute_fee_overflow_is_an_error_not_a_panic() {
    let (_, client, _, _) = setup();

    assert_eq!(
        client.try_compute_fee(&i128::MAX),
        Err(Ok(FeeError::Overflow))
    );
}

// -----------------------------------------------------------------------------
// Collection
// -----------------------------------------------------------------------------

#[test]
fn test_collect_fee_pulls_the_cut_and_credits_the_vault() {
    let (env, client, _, token) = setup();
    let payer = Address::generate(&env);
    mint(&env, &token, &payer, 10_000);

    let collected = client.collect_fee(&payer, &token, &10_000);

    assert_eq!(collected, 250);
    assert_eq!(client.get_balance(&token), 250);
    assert_eq!(TokenClient::new(&env, &token).balance(&payer), 9_750);
    assert_eq!(TokenClient::new(&env, &token).balance(&client.address), 250);
}

#[test]
fn test_collect_fee_at_zero_rate_is_a_no_op() {
    let (env, client, admin, token) = setup();
    client.set_fee_bps(&admin, &0);

    let payer = Address::generate(&env);
    mint(&env, &token, &payer, 10_000);

    assert_eq!(client.collect_fee(&payer, &token, &10_000), 0);
    assert_eq!(client.get_balance(&token), 0);
    assert_eq!(TokenClient::new(&env, &token).balance(&payer), 10_000);
    // A zero collection must not register the token as a fee source.
    assert_eq!(client.get_tokens().len(), 0);
}

#[test]
fn test_collect_fee_accumulates_across_calls() {
    let (env, client, _, token) = setup();
    let payer = Address::generate(&env);
    mint(&env, &token, &payer, 30_000);

    client.collect_fee(&payer, &token, &10_000);
    client.collect_fee(&payer, &token, &20_000);

    assert_eq!(client.get_balance(&token), 750);
    // The token is registered exactly once.
    assert_eq!(client.get_tokens().len(), 1);
}

#[test]
fn test_deposit_fee_rejects_non_positive_amounts() {
    let (env, client, _, token) = setup();
    let payer = Address::generate(&env);
    mint(&env, &token, &payer, 1_000);

    assert_eq!(
        client.try_deposit_fee(&payer, &token, &0),
        Err(Ok(FeeError::InvalidAmount))
    );
    assert_eq!(
        client.try_deposit_fee(&payer, &token, &-5),
        Err(Ok(FeeError::InvalidAmount))
    );
}

#[test]
fn test_get_balances_reports_every_collected_token() {
    let (env, client, _, token_a) = setup();
    let token_b = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    let payer = Address::generate(&env);
    mint(&env, &token_a, &payer, 10_000);
    mint(&env, &token_b, &payer, 40_000);

    client.collect_fee(&payer, &token_a, &10_000);
    client.collect_fee(&payer, &token_b, &40_000);

    let balances = client.get_balances();
    assert_eq!(balances.len(), 2);
    assert_eq!(balances.get_unchecked(0).token, token_a);
    assert_eq!(balances.get_unchecked(0).amount, 250);
    assert_eq!(balances.get_unchecked(1).token, token_b);
    assert_eq!(balances.get_unchecked(1).amount, 1_000);
}

// -----------------------------------------------------------------------------
// Withdrawal
// -----------------------------------------------------------------------------

#[test]
fn test_admin_can_withdraw_collected_fees() {
    let (env, client, admin, token) = setup();
    let payer = Address::generate(&env);
    let treasury = Address::generate(&env);
    mint(&env, &token, &payer, 10_000);
    client.collect_fee(&payer, &token, &10_000);

    client.withdraw_fees(&admin, &token, &treasury, &200);

    assert_eq!(TokenClient::new(&env, &token).balance(&treasury), 200);
    assert_eq!(client.get_balance(&token), 50);
}

#[test]
fn test_non_admin_cannot_withdraw() {
    let (env, client, _, token) = setup();
    let payer = Address::generate(&env);
    let thief = Address::generate(&env);
    mint(&env, &token, &payer, 10_000);
    client.collect_fee(&payer, &token, &10_000);

    assert_eq!(
        client.try_withdraw_fees(&thief, &token, &thief, &250),
        Err(Ok(FeeError::NotAuthorized))
    );
    assert_eq!(client.get_balance(&token), 250);
    assert_eq!(TokenClient::new(&env, &token).balance(&thief), 0);
}

#[test]
fn test_cannot_withdraw_more_than_collected() {
    let (env, client, admin, token) = setup();
    let payer = Address::generate(&env);
    mint(&env, &token, &payer, 10_000);
    client.collect_fee(&payer, &token, &10_000);

    assert_eq!(
        client.try_withdraw_fees(&admin, &token, &admin, &251),
        Err(Ok(FeeError::InsufficientBalance))
    );
    assert_eq!(client.get_balance(&token), 250);
}

#[test]
fn test_withdraw_rejects_non_positive_amounts() {
    let (_, client, admin, token) = setup();

    assert_eq!(
        client.try_withdraw_fees(&admin, &token, &admin, &0),
        Err(Ok(FeeError::InvalidAmount))
    );
}

#[test]
fn test_withdraw_cannot_drain_a_token_via_another_tokens_balance() {
    let (env, client, admin, token_a) = setup();
    let token_b = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    let payer = Address::generate(&env);
    mint(&env, &token_a, &payer, 10_000);
    client.collect_fee(&payer, &token_a, &10_000);

    // Nothing was ever collected in token B, so nothing is withdrawable in it.
    assert_eq!(
        client.try_withdraw_fees(&admin, &token_b, &admin, &1),
        Err(Ok(FeeError::InsufficientBalance))
    );
}
