//! Capability resolution tests.

use redclaw::core::contracts::{ExecutionPolicy, ModuleTrustTier};
use redclaw::core::security::evaluate_policy;

#[test]
fn deny_by_default_blocks_even_official_modules() {
    let policy = ExecutionPolicy::default();
    assert!(!evaluate_policy(&policy, ModuleTrustTier::Official));
}

#[test]
fn permissive_policy_allows_first_party_module() {
    let policy = ExecutionPolicy {
        safe_mode: false,
        deny_by_default: false,
        require_checksum: true,
        require_signature_for: vec![ModuleTrustTier::ThirdParty],
    };
    assert!(evaluate_policy(&policy, ModuleTrustTier::Reviewed));
}

#[test]
fn safe_mode_blocks_third_party_even_when_not_deny_by_default() {
    let policy = ExecutionPolicy {
        safe_mode: true,
        deny_by_default: false,
        require_checksum: true,
        require_signature_for: vec![ModuleTrustTier::ThirdParty],
    };
    assert!(!evaluate_policy(&policy, ModuleTrustTier::ThirdParty));
}
