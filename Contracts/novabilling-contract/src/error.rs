use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ContractError {
    AdminNotSet = 1,
    NotAuthorized = 2,
    SubscriptionNotFound = 3,
    SubscriptionNotActive = 4,
    BillingPeriodNotElapsed = 5,
    InvalidBillingParameters = 6,
    SubscriptionAlreadyTerminated = 7,
}
