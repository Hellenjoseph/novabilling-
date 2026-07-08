use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SubscriptionStatus {
    Active = 0,
    Paused = 1,
    Cancelled = 2,
    Delinquent = 3,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Subscription {
    pub subscriber: Address,
    pub merchant: Address,
    pub token: Address,
    pub rate: i128,
    pub period: u64,
    pub last_charge_timestamp: u64,
    pub status: SubscriptionStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum StorageKey {
    Admin,
    Subscription(u32),
    TotalSubscriptionsCount,
    SubscriberSubscriptions(Address),
    MerchantSubscriptions(Address),
}
