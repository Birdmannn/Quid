#![cfg(test)]

use super::*;
use quid_fee_collector::{QuidFeeCollectorContract, QuidFeeCollectorContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, Vec,
};

fn setup() -> (
    Env,
    QuidTreasuryMultisigContractClient<'static>,
    QuidFeeCollectorContractClient<'static>,
    Vec<Address>,
    Address,
) {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().with_mut(|ledger| ledger.timestamp = 1_000);

    let treasury_id = env.register(QuidTreasuryMultisigContract, ());
    let treasury = QuidTreasuryMultisigContractClient::new(&env, &treasury_id);
    let mut signers = Vec::new(&env);
    signers.push_back(Address::generate(&env));
    signers.push_back(Address::generate(&env));
    signers.push_back(Address::generate(&env));
    treasury.initialize(&signers, &2);

    let collector_id = env.register(QuidFeeCollectorContract, ());
    let collector = QuidFeeCollectorContractClient::new(&env, &collector_id);
    collector.initialize(&treasury_id, &250);

    let token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();
    (env, treasury, collector, signers, token)
}

fn fund_collector(
    env: &Env,
    collector: &QuidFeeCollectorContractClient,
    token: &Address,
    amount: i128,
) {
    let payer = Address::generate(env);
    StellarAssetClient::new(env, token).mint(&payer, &amount);
    collector.deposit_fee(&payer, token, &amount);
}

#[test]
fn test_threshold_requires_multiple_approvals_before_withdrawing_fees() {
    let (env, treasury, collector, signers, token) = setup();
    fund_collector(&env, &collector, &token, 500);
    let recipient = Address::generate(&env);
    let proposal_id = treasury.propose(
        &signers.get_unchecked(0),
        &collector.address,
        &token,
        &recipient,
        &300,
        &2_000,
    );

    treasury.approve(&signers.get_unchecked(0), &proposal_id);
    assert_eq!(
        treasury.try_execute_transfer(&proposal_id),
        Err(Ok(TreasuryError::InsufficientApprovals))
    );
    assert_eq!(TokenClient::new(&env, &token).balance(&recipient), 0);

    treasury.approve(&signers.get_unchecked(1), &proposal_id);
    treasury.execute_transfer(&proposal_id);

    assert_eq!(TokenClient::new(&env, &token).balance(&recipient), 300);
    assert_eq!(collector.get_balance(&token), 200);
    assert!(treasury.get_proposal(&proposal_id).executed);
}

#[test]
fn test_expired_proposal_cannot_be_approved_or_executed() {
    let (env, treasury, collector, signers, token) = setup();
    fund_collector(&env, &collector, &token, 100);
    let proposal_id = treasury.propose(
        &signers.get_unchecked(0),
        &collector.address,
        &token,
        &Address::generate(&env),
        &100,
        &1_010,
    );
    treasury.approve(&signers.get_unchecked(0), &proposal_id);
    treasury.approve(&signers.get_unchecked(1), &proposal_id);
    env.ledger().with_mut(|ledger| ledger.timestamp = 1_010);

    assert_eq!(
        treasury.try_execute_transfer(&proposal_id),
        Err(Ok(TreasuryError::ProposalExpired))
    );
    assert_eq!(
        treasury.try_approve(&signers.get_unchecked(2), &proposal_id),
        Err(Ok(TreasuryError::ProposalExpired))
    );
}

#[test]
fn test_duplicate_approval_and_invalid_threshold_are_rejected() {
    let (env, treasury, collector, signers, token) = setup();
    let proposal_id = treasury.propose(
        &signers.get_unchecked(0),
        &collector.address,
        &token,
        &Address::generate(&env),
        &1,
        &2_000,
    );
    treasury.approve(&signers.get_unchecked(0), &proposal_id);

    assert_eq!(
        treasury.try_approve(&signers.get_unchecked(0), &proposal_id),
        Err(Ok(TreasuryError::AlreadyApproved))
    );
    assert_eq!(
        treasury.try_set_threshold(&0),
        Err(Ok(TreasuryError::InvalidThreshold))
    );
    assert_eq!(
        treasury.try_set_threshold(&4),
        Err(Ok(TreasuryError::InvalidThreshold))
    );
}
