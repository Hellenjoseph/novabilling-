#![no_std]

pub mod error;
pub mod helper;
pub mod storage;
#[cfg(test)]
pub mod test;
pub mod types;

use crate::error::ContractError;
use crate::types::{Subscription, SubscriptionStatus};
use soroban_sdk::{contract, contractimpl, Address, Env, Vec};

#[contract]
pub struct NovaBillingContract;

#[contractimpl]
impl NovaBillingContract {
    /// Sets the admin of the billing contract. Can only be set once.
    pub fn set_admin(env: Env, admin: Address) -> Result<(), ContractError> {
        if let Some(current_admin) = storage::get_admin(&env) {
            current_admin.require_auth();
        }
        storage::set_admin(&env, &admin);
        Ok(())
    }

    /// Creates a recurring subscription and executes the first billing cycle immediately.
    pub fn create_subscription(
        env: Env,
        subscriber: Address,
        merchant: Address,
        token: Address,
        rate: i128,
        period: u64,
    ) -> Result<u32, ContractError> {
        subscriber.require_auth();

        if rate <= 0 || period == 0 {
            return Err(ContractError::InvalidBillingParameters);
        }

        // Charge the first billing cycle immediately
        let token_client = soroban_sdk::token::Client::new(&env, &token);
        token_client.transfer_from(
            &env.current_contract_address(),
            &subscriber,
            &merchant,
            &rate,
        );

        let sub = Subscription {
            subscriber: subscriber.clone(),
            merchant: merchant.clone(),
            token,
            rate,
            period,
            last_charge_timestamp: env.ledger().timestamp(),
            status: SubscriptionStatus::Active,
        };

        let sub_id = storage::increment_total_subscriptions(&env);
        storage::set_subscription(&env, sub_id, &sub);
        storage::add_subscriber_subscription(&env, &subscriber, sub_id);
        storage::add_merchant_subscription(&env, &merchant, sub_id);

        Ok(sub_id)
    }

    /// Charges the subscriber's wallet for the elapsed billing period (invoked by Merchant/Keepers).
    pub fn charge_subscription(env: Env, subscription_id: u32) -> Result<(), ContractError> {
        let mut sub = storage::get_subscription(&env, subscription_id)
            .ok_or(ContractError::SubscriptionNotFound)?;

        // Enforce active status
        if sub.status != SubscriptionStatus::Active && sub.status != SubscriptionStatus::Delinquent
        {
            return Err(ContractError::SubscriptionNotActive);
        }

        // Check if billing cycle elapsed
        if !helper::is_billing_period_elapsed(&env, sub.last_charge_timestamp, sub.period) {
            return Err(ContractError::BillingPeriodNotElapsed);
        }

        // Check balance and allowance to prevent transaction failures
        let token_client = soroban_sdk::token::Client::new(&env, &sub.token);
        let balance = token_client.balance(&sub.subscriber);
        let allowance = token_client.allowance(&sub.subscriber, &env.current_contract_address());

        if balance < sub.rate || allowance < sub.rate {
            // Subscription enters Delinquent status
            sub.status = SubscriptionStatus::Delinquent;
            storage::set_subscription(&env, subscription_id, &sub);
            return Ok(());
        }

        // Pull payment
        token_client.transfer_from(
            &env.current_contract_address(),
            &sub.subscriber,
            &sub.merchant,
            &sub.rate,
        );

        // Update charge timestamps and reset active status
        sub.last_charge_timestamp = env.ledger().timestamp();
        sub.status = SubscriptionStatus::Active;
        storage::set_subscription(&env, subscription_id, &sub);

        Ok(())
    }

    /// Pauses billing charge cycles (invoked by Subscriber).
    pub fn pause_subscription(env: Env, subscription_id: u32) -> Result<(), ContractError> {
        let mut sub = storage::get_subscription(&env, subscription_id)
            .ok_or(ContractError::SubscriptionNotFound)?;

        if sub.status != SubscriptionStatus::Active {
            return Err(ContractError::SubscriptionNotActive);
        }

        sub.subscriber.require_auth();

        sub.status = SubscriptionStatus::Paused;
        storage::set_subscription(&env, subscription_id, &sub);

        Ok(())
    }

    /// Resumes billing charge cycles (invoked by Subscriber).
    pub fn resume_subscription(env: Env, subscription_id: u32) -> Result<(), ContractError> {
        let mut sub = storage::get_subscription(&env, subscription_id)
            .ok_or(ContractError::SubscriptionNotFound)?;

        if sub.status != SubscriptionStatus::Paused {
            return Err(ContractError::SubscriptionNotActive);
        }

        sub.subscriber.require_auth();

        sub.status = SubscriptionStatus::Active;
        // Resume updates billing timeline from the current ledger time to prevent retroactive double-billing
        sub.last_charge_timestamp = env.ledger().timestamp();
        storage::set_subscription(&env, subscription_id, &sub);

        Ok(())
    }

    /// Cancels subscription billing permanently (invoked by Subscriber or Merchant).
    pub fn cancel_subscription(
        env: Env,
        caller: Address,
        subscription_id: u32,
    ) -> Result<(), ContractError> {
        caller.require_auth();

        let mut sub = storage::get_subscription(&env, subscription_id)
            .ok_or(ContractError::SubscriptionNotFound)?;

        if sub.status == SubscriptionStatus::Cancelled {
            return Err(ContractError::SubscriptionAlreadyTerminated);
        }

        // Only subscriber or merchant can cancel
        if caller != sub.subscriber && caller != sub.merchant {
            return Err(ContractError::NotAuthorized);
        }

        sub.status = SubscriptionStatus::Cancelled;
        storage::set_subscription(&env, subscription_id, &sub);

        Ok(())
    }

    /// Reads subscription details.
    pub fn get_subscription(env: Env, subscription_id: u32) -> Option<Subscription> {
        storage::get_subscription(&env, subscription_id)
    }

    /// Reads subscriber subscriptions.
    pub fn get_subscriber_subscriptions(env: Env, subscriber: Address) -> Vec<u32> {
        storage::get_subscriber_subscriptions(&env, &subscriber)
    }

    /// Reads merchant subscriptions.
    pub fn get_merchant_subscriptions(env: Env, merchant: Address) -> Vec<u32> {
        storage::get_merchant_subscriptions(&env, &merchant)
    }
}
