//! Module health status.

use serde::{Deserialize, Serialize};

/// Health status of a module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleHealth {
    /// Module ID.
    pub module_id: String,
    /// Whether the module is healthy.
    pub healthy: bool,
    /// Last error message, if any.
    pub last_error: Option<String>,
}
