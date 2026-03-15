//! Safe mode baseline modules tests.

use redclaw::core::contracts::{ActivationPhase, ActivationResult};
use redclaw::core::lifecycle::{
    is_baseline_module, verify_baseline_modules, SAFE_MODE_BASELINE_MODULES,
};

#[test]
fn safe_mode_baseline_contains_required_modules_only() {
    assert!(SAFE_MODE_BASELINE_MODULES.contains(&"runtime-native"));
    assert!(SAFE_MODE_BASELINE_MODULES.contains(&"channel-cli"));
    assert!(!SAFE_MODE_BASELINE_MODULES.contains(&"provider-openai-compatible"));
    assert!(!SAFE_MODE_BASELINE_MODULES.contains(&"tool-shell"));
}

#[test]
fn baseline_module_membership_helpers_are_consistent() {
    assert!(is_baseline_module("runtime-native"));
    assert!(is_baseline_module("channel-cli"));
    assert!(!is_baseline_module("tool-shell"));
}

#[test]
fn verify_baseline_modules_requires_success_for_every_baseline_module() {
    let partial = vec![ActivationResult {
        module_id: "runtime-native".to_string(),
        phase: ActivationPhase::Activated,
        success: true,
        diagnostics: vec![],
    }];
    assert!(!verify_baseline_modules(&partial));

    let complete = vec![
        ActivationResult {
            module_id: "runtime-native".to_string(),
            phase: ActivationPhase::Activated,
            success: true,
            diagnostics: vec![],
        },
        ActivationResult {
            module_id: "channel-cli".to_string(),
            phase: ActivationPhase::Activated,
            success: true,
            diagnostics: vec![],
        },
    ];
    assert!(verify_baseline_modules(&complete));
}
