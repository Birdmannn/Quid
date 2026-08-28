#![cfg(test)]

use super::*;
use soroban_sdk::{
    contract as test_contract, contractimpl as test_contractimpl,
    testutils::Address as _,
    token::{Client as TokenClient, StellarAssetClient},
    Address, Env, String,
};

const REWARD_BPS: u32 = 500; // 5%
const PAYOUT: i128 = 10_000; // reward on this payout: 500

struct Fixture<'a> {
    env: Env,
    client: QuidReferralContractClient<'a>,
    admin: Address,
    hook: Address,
    token: Address,
}

fn setup() -> Fixture<'static> {
    let env = Env::default();
    env.mock_all_auths();

    let client = QuidReferralContractClient::new(&env, &env.register(QuidReferralContract, ()));

    let admin = Address::generate(&env);
    client.initialize(&admin, &REWARD_BPS);

    // A plain address stands in for the deployed quid-store contract.
    let hook = Address::generate(&env);
    client.set_payout_hook(&admin, &hook);

    let token = env
        .register_stellar_asset_contract_v2(Address::generate(&env))
        .address();

    Fixture {
        env,
        client,
        admin,
        hook,
        token,
    }
}

fn mint(env: &Env, token: &Address, to: &Address, amount: i128) {
    StellarAssetClient::new(env, token).mint(to, &amount);
}

/// Register `referred` under `referrer` and fund the pool so a claim can settle.
fn referral_ready(f: &Fixture) -> (Address, Address) {
    let referrer = Address::generate(&f.env);
    let referred = Address::generate(&f.env);

    f.client.register_referral(
        &referrer,
        &referred,
        &String::from_str(&f.env, "QmReferralLink"),
    );

    let sponsor = Address::generate(&f.env);
    mint(&f.env, &f.token, &sponsor, 1_000_000);
    f.client.fund(&sponsor, &f.token, &1_000_000);

    (referrer, referred)
}

// -----------------------------------------------------------------------------
// Bootstrap
// -----------------------------------------------------------------------------

#[test]
fn test_initialize_sets_admin_and_rate() {
    let f = setup();

    assert_eq!(f.client.get_admin(), f.admin);
    assert_eq!(f.client.get_reward_bps(), REWARD_BPS);
    assert_eq!(f.client.get_payout_hook(), f.hook);
}

#[test]
fn test_initialize_twice_fails() {
    let f = setup();
    let other = Address::generate(&f.env);

    assert_eq!(
        f.client.try_initialize(&other, &100),
        Err(Ok(ReferralError::AlreadyInitialized))
    );
}

#[test]
fn test_initialize_rejects_rate_above_one_hundred_percent() {
    let env = Env::default();
    env.mock_all_auths();
    let client = QuidReferralContractClient::new(&env, &env.register(QuidReferralContract, ()));

    assert_eq!(
        client.try_initialize(&Address::generate(&env), &(MAX_REWARD_BPS + 1)),
        Err(Ok(ReferralError::InvalidRewardBps))
    );
}

#[test]
fn test_set_admin_transfers_control() {
    let f = setup();
    let new_admin = Address::generate(&f.env);

    f.client.set_admin(&f.admin, &new_admin);

    assert_eq!(f.client.get_admin(), new_admin);
    assert_eq!(
        f.client.try_set_reward_bps(&f.admin, &10),
        Err(Ok(ReferralError::NotAuthorized))
    );
}

// -----------------------------------------------------------------------------
// Reward rate
// -----------------------------------------------------------------------------

#[test]
fn test_admin_can_update_reward_bps() {
    let f = setup();

    f.client.set_reward_bps(&f.admin, &1_000);

    assert_eq!(f.client.get_reward_bps(), 1_000);
}

#[test]
fn test_non_admin_cannot_update_reward_bps() {
    let f = setup();
    let stranger = Address::generate(&f.env);

    assert_eq!(
        f.client.try_set_reward_bps(&stranger, &0),
        Err(Ok(ReferralError::NotAuthorized))
    );
    assert_eq!(f.client.get_reward_bps(), REWARD_BPS);
}

#[test]
fn test_set_reward_bps_rejects_rate_above_one_hundred_percent() {
    let f = setup();

    assert_eq!(
        f.client.try_set_reward_bps(&f.admin, &(MAX_REWARD_BPS + 1)),
        Err(Ok(ReferralError::InvalidRewardBps))
    );
    assert_eq!(f.client.get_reward_bps(), REWARD_BPS);
}

#[test]
fn test_non_admin_cannot_move_the_payout_hook() {
    let f = setup();
    let attacker = Address::generate(&f.env);

    assert_eq!(
        f.client.try_set_payout_hook(&attacker, &attacker),
        Err(Ok(ReferralError::NotAuthorized))
    );
    assert_eq!(f.client.get_payout_hook(), f.hook);
}

#[test]
fn test_compute_reward_math() {
    let f = setup();

    assert_eq!(f.client.compute_reward(&10_000), 500);
    assert_eq!(f.client.compute_reward(&0), 0);
    // Rounds down: 5% of 19 == 0.95
    assert_eq!(f.client.compute_reward(&19), 0);

    f.client.set_reward_bps(&f.admin, &0);
    assert_eq!(f.client.compute_reward(&10_000), 0);
}

#[test]
fn test_compute_reward_rejects_negative_and_overflow() {
    let f = setup();

    assert_eq!(
        f.client.try_compute_reward(&-1),
        Err(Ok(ReferralError::InvalidAmount))
    );
    assert_eq!(
        f.client.try_compute_reward(&i128::MAX),
        Err(Ok(ReferralError::Overflow))
    );
}

// -----------------------------------------------------------------------------
// Registration
// -----------------------------------------------------------------------------

#[test]
fn test_register_referral_records_link_and_hash() {
    let f = setup();
    let referrer = Address::generate(&f.env);
    let referred = Address::generate(&f.env);
    let cid = String::from_str(&f.env, "QmReferralLink");

    f.client.register_referral(&referrer, &referred, &cid);

    let referral = f.client.get_referral(&referred);
    assert_eq!(referral.referrer, referrer);
    assert_eq!(referral.referred, referred);
    assert_eq!(referral.link_cid, cid);
    assert_eq!(
        referral.link_hash,
        f.env.crypto().sha256(&cid.to_xdr(&f.env)).to_bytes()
    );
    assert!(f.client.has_referral(&referred));
    assert_eq!(f.client.get_referred_count(&referrer), 1);
}

#[test]
fn test_a_hunter_can_only_be_referred_once() {
    let f = setup();
    let first = Address::generate(&f.env);
    let second = Address::generate(&f.env);
    let referred = Address::generate(&f.env);
    let cid = String::from_str(&f.env, "QmReferralLink");

    f.client.register_referral(&first, &referred, &cid);

    // A second referrer cannot overwrite the attribution.
    assert_eq!(
        f.client.try_register_referral(&second, &referred, &cid),
        Err(Ok(ReferralError::AlreadyRegistered))
    );
    assert_eq!(f.client.get_referral(&referred).referrer, first);
    assert_eq!(f.client.get_referred_count(&second), 0);
}

#[test]
fn test_self_referral_is_rejected() {
    let f = setup();
    let hunter = Address::generate(&f.env);

    assert_eq!(
        f.client.try_register_referral(
            &hunter,
            &hunter,
            &String::from_str(&f.env, "QmReferralLink")
        ),
        Err(Ok(ReferralError::SelfReferral))
    );
}

#[test]
fn test_empty_link_cid_is_rejected() {
    let f = setup();

    assert_eq!(
        f.client.try_register_referral(
            &Address::generate(&f.env),
            &Address::generate(&f.env),
            &String::from_str(&f.env, "")
        ),
        Err(Ok(ReferralError::InvalidInput))
    );
}

#[test]
fn test_get_referral_for_unknown_hunter() {
    let f = setup();

    assert_eq!(
        f.client.try_get_referral(&Address::generate(&f.env)),
        Err(Ok(ReferralError::ReferralNotFound))
    );
}

#[test]
fn test_one_referrer_can_refer_many_hunters() {
    let f = setup();
    let referrer = Address::generate(&f.env);
    let cid = String::from_str(&f.env, "QmReferralLink");

    for _ in 0..3 {
        f.client
            .register_referral(&referrer, &Address::generate(&f.env), &cid);
    }

    assert_eq!(f.client.get_referred_count(&referrer), 3);
}

// -----------------------------------------------------------------------------
// Accrual is gated behind a successful payout
// -----------------------------------------------------------------------------

#[test]
fn test_nothing_is_claimable_before_a_payout() {
    let f = setup();
    let (referrer, _) = referral_ready(&f);

    assert_eq!(f.client.get_claimable(&referrer, &f.token), 0);
    assert_eq!(
        f.client.try_claim_reward(&referrer, &f.token),
        Err(Ok(ReferralError::NothingToClaim))
    );
}

#[test]
fn test_record_payout_accrues_the_referrers_cut() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);

    let accrued = f
        .client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);

    assert_eq!(accrued, 500);
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 500);
    assert!(f.client.is_payout_recorded(&1, &referred));
}

#[test]
fn test_only_the_payout_hook_can_record_a_payout() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);
    let attacker = Address::generate(&f.env);

    assert_eq!(
        f.client
            .try_record_payout(&attacker, &referred, &1, &f.token, &PAYOUT),
        Err(Ok(ReferralError::NotAuthorized))
    );
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 0);
    assert!(!f.client.is_payout_recorded(&1, &referred));
}

#[test]
fn test_record_payout_for_an_unreferred_hunter_is_a_no_op() {
    let f = setup();
    let stranger = Address::generate(&f.env);

    assert_eq!(
        f.client
            .record_payout(&f.hook, &stranger, &1, &f.token, &PAYOUT),
        0
    );
}

#[test]
fn test_a_referral_registered_after_a_payout_cannot_be_back_paid() {
    let f = setup();
    let referrer = Address::generate(&f.env);
    let referred = Address::generate(&f.env);

    // Paid first, referral registered afterwards.
    f.client
        .record_payout(&f.hook, &referred, &7, &f.token, &PAYOUT);
    f.client.register_referral(
        &referrer,
        &referred,
        &String::from_str(&f.env, "QmReferralLink"),
    );

    assert_eq!(
        f.client
            .try_record_payout(&f.hook, &referred, &7, &f.token, &PAYOUT),
        Err(Ok(ReferralError::PayoutAlreadyRecorded))
    );
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 0);
}

#[test]
fn test_record_payout_rejects_non_positive_amounts() {
    let f = setup();
    let (_, referred) = referral_ready(&f);

    assert_eq!(
        f.client
            .try_record_payout(&f.hook, &referred, &1, &f.token, &0),
        Err(Ok(ReferralError::InvalidAmount))
    );
    assert_eq!(
        f.client
            .try_record_payout(&f.hook, &referred, &1, &f.token, &-1),
        Err(Ok(ReferralError::InvalidAmount))
    );
}

#[test]
fn test_accrual_accumulates_across_missions() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);
    f.client
        .record_payout(&f.hook, &referred, &2, &f.token, &PAYOUT);

    assert_eq!(f.client.get_claimable(&referrer, &f.token), 1_000);
}

#[test]
fn test_accrual_is_tracked_per_token() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);
    let other_token = f
        .env
        .register_stellar_asset_contract_v2(Address::generate(&f.env))
        .address();

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);
    f.client
        .record_payout(&f.hook, &referred, &2, &other_token, &PAYOUT);

    assert_eq!(f.client.get_claimable(&referrer, &f.token), 500);
    assert_eq!(f.client.get_claimable(&referrer, &other_token), 500);
}

#[test]
fn test_rate_change_does_not_retroactively_reprice_accrued_rewards() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);
    f.client.set_reward_bps(&f.admin, &0);
    f.client
        .record_payout(&f.hook, &referred, &2, &f.token, &PAYOUT);

    assert_eq!(f.client.get_claimable(&referrer, &f.token), 500);
}

// -----------------------------------------------------------------------------
// Double-claim prevention
// -----------------------------------------------------------------------------

#[test]
fn test_the_same_payout_cannot_be_recorded_twice() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);

    assert_eq!(
        f.client
            .try_record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT),
        Err(Ok(ReferralError::PayoutAlreadyRecorded))
    );
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 500);
}

#[test]
fn test_replaying_a_payout_in_another_token_does_not_double_pay() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);
    let other_token = f
        .env
        .register_stellar_asset_contract_v2(Address::generate(&f.env))
        .address();

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);

    // The dedupe key is (mission, hunter) — swapping the token does not unlock it.
    assert_eq!(
        f.client
            .try_record_payout(&f.hook, &referred, &1, &other_token, &PAYOUT),
        Err(Ok(ReferralError::PayoutAlreadyRecorded))
    );
    assert_eq!(f.client.get_claimable(&referrer, &other_token), 0);
}

#[test]
fn test_claim_pays_once_and_leaves_nothing_behind() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);
    let token_client = TokenClient::new(&f.env, &f.token);

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);

    assert_eq!(f.client.claim_reward(&referrer, &f.token), 500);
    assert_eq!(token_client.balance(&referrer), 500);
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 0);
    assert_eq!(f.client.get_claimed(&referrer, &f.token), 500);

    // Second claim finds an empty balance.
    assert_eq!(
        f.client.try_claim_reward(&referrer, &f.token),
        Err(Ok(ReferralError::NothingToClaim))
    );
    assert_eq!(token_client.balance(&referrer), 500);
}

#[test]
fn test_claiming_again_after_a_new_payout_only_pays_the_new_accrual() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);
    let token_client = TokenClient::new(&f.env, &f.token);

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);
    f.client.claim_reward(&referrer, &f.token);

    f.client
        .record_payout(&f.hook, &referred, &2, &f.token, &PAYOUT);
    assert_eq!(f.client.claim_reward(&referrer, &f.token), 500);

    assert_eq!(token_client.balance(&referrer), 1_000);
    assert_eq!(f.client.get_claimed(&referrer, &f.token), 1_000);
}

#[test]
fn test_one_referrers_balance_is_not_claimable_by_another() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);
    let stranger = Address::generate(&f.env);

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);

    assert_eq!(
        f.client.try_claim_reward(&stranger, &f.token),
        Err(Ok(ReferralError::NothingToClaim))
    );
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 500);
}

#[test]
fn test_claim_in_a_token_with_no_accrual() {
    let f = setup();
    let (referrer, referred) = referral_ready(&f);
    let other_token = f
        .env
        .register_stellar_asset_contract_v2(Address::generate(&f.env))
        .address();

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);

    assert_eq!(
        f.client.try_claim_reward(&referrer, &other_token),
        Err(Ok(ReferralError::NothingToClaim))
    );
}

// -----------------------------------------------------------------------------
// Reward pool
// -----------------------------------------------------------------------------

#[test]
fn test_claim_fails_when_the_pool_is_short() {
    let f = setup();
    let referrer = Address::generate(&f.env);
    let referred = Address::generate(&f.env);
    f.client.register_referral(
        &referrer,
        &referred,
        &String::from_str(&f.env, "QmReferralLink"),
    );

    // Pool holds 100, the accrual is 500.
    let sponsor = Address::generate(&f.env);
    mint(&f.env, &f.token, &sponsor, 100);
    f.client.fund(&sponsor, &f.token, &100);

    f.client
        .record_payout(&f.hook, &referred, &1, &f.token, &PAYOUT);

    assert_eq!(
        f.client.try_claim_reward(&referrer, &f.token),
        Err(Ok(ReferralError::InsufficientFunds))
    );
    // The balance survives the failed claim and settles once the pool is topped up.
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 500);

    mint(&f.env, &f.token, &sponsor, 400);
    f.client.fund(&sponsor, &f.token, &400);
    assert_eq!(f.client.claim_reward(&referrer, &f.token), 500);
}

#[test]
fn test_fund_rejects_non_positive_amounts() {
    let f = setup();
    let sponsor = Address::generate(&f.env);

    assert_eq!(
        f.client.try_fund(&sponsor, &f.token, &0),
        Err(Ok(ReferralError::InvalidAmount))
    );
    assert_eq!(
        f.client.try_fund(&sponsor, &f.token, &-1),
        Err(Ok(ReferralError::InvalidAmount))
    );
}

#[test]
fn test_fund_tops_up_the_pool() {
    let f = setup();
    let sponsor = Address::generate(&f.env);
    mint(&f.env, &f.token, &sponsor, 1_000);

    f.client.fund(&sponsor, &f.token, &750);

    assert_eq!(f.client.get_pool_balance(&f.token), 750);
    assert_eq!(TokenClient::new(&f.env, &f.token).balance(&sponsor), 250);
}

// -----------------------------------------------------------------------------
// The hook as another contract calls it
// -----------------------------------------------------------------------------

/// Stand-in for `quid-store`: reports a payout the way the store will once the
/// payout hook lands (issue #290).
#[test_contract]
pub struct MockStore;

#[test_contractimpl]
impl MockStore {
    pub fn pay(
        env: Env,
        referral: Address,
        hunter: Address,
        mission_id: u64,
        token: Address,
        amount: i128,
    ) -> i128 {
        QuidReferralContractClient::new(&env, &referral).record_payout(
            &env.current_contract_address(),
            &hunter,
            &mission_id,
            &token,
            &amount,
        )
    }
}

#[test]
fn test_a_contract_hook_can_report_payouts() {
    let f = setup();
    let store = f.env.register(MockStore, ());
    f.client.set_payout_hook(&f.admin, &store);

    let (referrer, referred) = referral_ready(&f);

    let accrued = MockStoreClient::new(&f.env, &store).pay(
        &f.client.address,
        &referred,
        &1,
        &f.token,
        &PAYOUT,
    );

    assert_eq!(accrued, 500);
    assert_eq!(f.client.get_claimable(&referrer, &f.token), 500);
    assert_eq!(f.client.claim_reward(&referrer, &f.token), 500);
}

#[test]
fn test_a_contract_that_is_not_the_hook_cannot_report_payouts() {
    let f = setup();
    let rogue = f.env.register(MockStore, ());
    let (_, referred) = referral_ready(&f);

    let result = MockStoreClient::new(&f.env, &rogue).try_pay(
        &f.client.address,
        &referred,
        &1,
        &f.token,
        &PAYOUT,
    );

    // The referral contract rejects it, which unwinds the calling contract too.
    assert!(result.is_err());
    assert!(!f.client.is_payout_recorded(&1, &referred));
}
