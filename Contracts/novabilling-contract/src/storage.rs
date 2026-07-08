use crate::types::{StorageKey, Subscription};
use soroban_sdk::{Address, Env, Vec};

pub fn get_admin(env: &Env) -> Option<Address> {
    env.storage().persistent().get(&StorageKey::Admin)
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().persistent().set(&StorageKey::Admin, admin);
}

pub fn get_subscription(env: &Env, sub_id: u32) -> Option<Subscription> {
    env.storage()
        .persistent()
        .get(&StorageKey::Subscription(sub_id))
}

pub fn set_subscription(env: &Env, sub_id: u32, sub: &Subscription) {
    env.storage()
        .persistent()
        .set(&StorageKey::Subscription(sub_id), sub);
}

pub fn get_total_subscriptions(env: &Env) -> u32 {
    env.storage()
        .persistent()
        .get(&StorageKey::TotalSubscriptionsCount)
        .unwrap_or(0)
}

pub fn increment_total_subscriptions(env: &Env) -> u32 {
    let count = get_total_subscriptions(env) + 1;
    env.storage()
        .persistent()
        .set(&StorageKey::TotalSubscriptionsCount, &count);
    count
}

pub fn get_subscriber_subscriptions(env: &Env, subscriber: &Address) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&StorageKey::SubscriberSubscriptions(subscriber.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_subscriber_subscription(env: &Env, subscriber: &Address, sub_id: u32) {
    let mut list = get_subscriber_subscriptions(env, subscriber);
    list.push_back(sub_id);
    env.storage().persistent().set(
        &StorageKey::SubscriberSubscriptions(subscriber.clone()),
        &list,
    );
}

pub fn get_merchant_subscriptions(env: &Env, merchant: &Address) -> Vec<u32> {
    env.storage()
        .persistent()
        .get(&StorageKey::MerchantSubscriptions(merchant.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn add_merchant_subscription(env: &Env, merchant: &Address, sub_id: u32) {
    let mut list = get_merchant_subscriptions(env, merchant);
    list.push_back(sub_id);
    env.storage()
        .persistent()
        .set(&StorageKey::MerchantSubscriptions(merchant.clone()), &list);
}
