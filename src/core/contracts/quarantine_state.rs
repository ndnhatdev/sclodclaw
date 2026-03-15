//! Quarantine state enumeration.

use serde::{Deserialize, Serialize};

/// Quarantine state for a module.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum QuarantineState {
    /// Module is clear to execute.
    Clear,
    /// Module is quarantined (blocked from execution).
    Quarantined,
}
