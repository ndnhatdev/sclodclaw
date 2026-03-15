//! Policy evaluation.

use crate::core::contracts::{ExecutionPolicy, ModuleTrustTier};

pub fn evaluate_policy(policy: &ExecutionPolicy, trust_tier: ModuleTrustTier) -> bool {
    if policy.safe_mode && trust_tier == ModuleTrustTier::ThirdParty {
        return false;
    }
    if policy.deny_by_default {
        return false;
    }
    true
}
