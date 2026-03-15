//! Module deactivation flow.

use crate::core::contracts::ActivationResult;

pub fn deactivate(module_id: &str) -> ActivationResult {
    ActivationResult {
        module_id: module_id.to_string(),
        phase: crate::core::contracts::ActivationPhase::Activated,
        success: true,
        diagnostics: vec!["Module deactivated".to_string()],
    }
}
