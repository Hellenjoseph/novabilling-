use soroban_sdk::Env;

/// Returns true if the current ledger timestamp has surpassed the next scheduled billing date.
pub fn is_billing_period_elapsed(env: &Env, last_charge_timestamp: u64, period: u64) -> bool {
    let now = env.ledger().timestamp();
    now >= last_charge_timestamp + period
}
