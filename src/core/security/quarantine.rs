//! Quarantine enforcement.

use crate::core::contracts::QuarantineState;

pub fn quarantine_module(id: &str) -> QuarantineState {
    debug_assert!(
        !id.trim().is_empty(),
        "quarantine_module expects a module id"
    );
    QuarantineState::Quarantined
}

pub fn release_module(id: &str) -> QuarantineState {
    debug_assert!(!id.trim().is_empty(), "release_module expects a module id");
    QuarantineState::Clear
}
