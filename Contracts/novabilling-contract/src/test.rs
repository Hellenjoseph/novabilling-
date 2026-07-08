#![cfg(test)]

use crate::types::SubscriptionStatus;
use crate::{NovaBillingContract, NovaBillingContractClient};
use soroban_sdk::{
    testutils::{Address as _, Ledger},
    token::StellarAssetClient,
    Address, Env,
};

fn setup_test_env(env: &Env) -> (NovaBillingContractClient<'_>, Address, Address, Address) {
    env.mock_all_auths();

    // Register contract
    let contract_id = env.register(NovaBillingContract, ());
    let client = NovaBillingContractClient::new(env, &contract_id);

    // Generate accounts
    let admin = Address::generate(env);
    let subscriber = Address::generate(env);
    let merchant = Address::generate(env);

    client.set_admin(&admin);

    (client, admin, subscriber, merchant)
}

fn create_mock_token(env: &Env, admin: &Address) -> Address {
    env.register_stellar_asset_contract_v2(admin.clone())
        .address()
}

#[test]
fn test_subscription_creation_and_first_charge() {
    let env = Env::default();
    let (client, _, subscriber, merchant) = setup_test_env(&env);
    let token_admin = Address::generate(&env);
    let token_address = create_mock_token(&env, &token_admin);

    // Setup balances
    let sac_client = StellarAssetClient::new(&env, &token_address);
    sac_client.mint(&subscriber, &10000);

    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    token_client.approve(&subscriber, &client.address, &10000, &100000);

    // Create subscription (15 USDC rate, 30 days period)
    let period = 2592000; // 30 days in seconds
    let sub_id = client.create_subscription(&subscriber, &merchant, &token_address, &15, &period);

    // Verify first charge is executed immediately (15 tokens transferred)
    assert_eq!(token_client.balance(&subscriber), 9985);
    assert_eq!(token_client.balance(&merchant), 15);

    let sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Active);
    assert_eq!(sub.last_charge_timestamp, env.ledger().timestamp());
}

#[test]
fn test_successful_billing_cycle() {
    let env = Env::default();
    let (client, _, subscriber, merchant) = setup_test_env(&env);
    let token_admin = Address::generate(&env);
    let token_address = create_mock_token(&env, &token_admin);

    let sac_client = StellarAssetClient::new(&env, &token_address);
    sac_client.mint(&subscriber, &10000);
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    token_client.approve(&subscriber, &client.address, &10000, &100000);

    let period = 2592000;
    let sub_id = client.create_subscription(&subscriber, &merchant, &token_address, &20, &period);

    // Fast forward ledger time by 30 days
    let next_billing_time = env.ledger().timestamp() + period;
    env.ledger().set_timestamp(next_billing_time);

    // Execute second charge cycle
    client.charge_subscription(&sub_id);

    // Verify balance changes (subscriber charged 20 more, total 40 charged)
    assert_eq!(token_client.balance(&subscriber), 9960);
    assert_eq!(token_client.balance(&merchant), 40);

    let sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(sub.last_charge_timestamp, next_billing_time);
}

#[test]
fn test_double_charge_protection() {
    let env = Env::default();
    let (client, _, subscriber, merchant) = setup_test_env(&env);
    let token_admin = Address::generate(&env);
    let token_address = create_mock_token(&env, &token_admin);

    let sac_client = StellarAssetClient::new(&env, &token_address);
    sac_client.mint(&subscriber, &10000);
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    token_client.approve(&subscriber, &client.address, &10000, &100000);

    let period = 2592000;
    let sub_id = client.create_subscription(&subscriber, &merchant, &token_address, &20, &period);

    // Fast forward ledger time by only 15 days (cycle not complete)
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + 1296000);

    // Try to charge (should fail with BillingPeriodNotElapsed)
    let res = client.try_charge_subscription(&sub_id);
    assert!(res.is_err());
}

#[test]
fn test_delinquency_transition() {
    let env = Env::default();
    let (client, _, subscriber, merchant) = setup_test_env(&env);
    let token_admin = Address::generate(&env);
    let token_address = create_mock_token(&env, &token_admin);

    let sac_client = StellarAssetClient::new(&env, &token_address);
    sac_client.mint(&subscriber, &20); // only has enough for the first charge
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    token_client.approve(&subscriber, &client.address, &10000, &100000);

    let period = 2592000;
    let sub_id = client.create_subscription(&subscriber, &merchant, &token_address, &20, &period);

    // Fast forward 30 days
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + period);

    // Try to charge subscription (subscriber has 0 balance now, should execute and mark delinquent)
    client.charge_subscription(&sub_id);

    // Verify subscription enters Delinquent status
    let sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Delinquent);
}

#[test]
fn test_pause_and_cancel_controls() {
    let env = Env::default();
    let (client, _, subscriber, merchant) = setup_test_env(&env);
    let token_admin = Address::generate(&env);
    let token_address = create_mock_token(&env, &token_admin);

    let sac_client = StellarAssetClient::new(&env, &token_address);
    sac_client.mint(&subscriber, &10000);
    let token_client = soroban_sdk::token::Client::new(&env, &token_address);
    token_client.approve(&subscriber, &client.address, &10000, &100000);

    let period = 2592000;
    let sub_id = client.create_subscription(&subscriber, &merchant, &token_address, &20, &period);

    // Pause subscription
    client.pause_subscription(&sub_id);
    let sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(sub.status, SubscriptionStatus::Paused);

    // Fast forward 30 days and try to charge (should reject)
    env.ledger()
        .set_timestamp(env.ledger().timestamp() + period);
    let charge_res = client.try_charge_subscription(&sub_id);
    assert!(charge_res.is_err());

    // Resume subscription
    client.resume_subscription(&sub_id);
    let resumed_sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(resumed_sub.status, SubscriptionStatus::Active);

    // Cancel subscription
    client.cancel_subscription(&subscriber, &sub_id);
    let cancelled_sub = client.get_subscription(&sub_id).unwrap();
    assert_eq!(cancelled_sub.status, SubscriptionStatus::Cancelled);

    // Charge should reject permanently
    let charge_after_cancel = client.try_charge_subscription(&sub_id);
    assert!(charge_after_cancel.is_err());
}
