//! Capability grant policy.

use serde::{Deserialize, Serialize};

/// Policy for granting capabilities to modules.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct CapabilityGrantPolicy {
    /// Whether to grant capabilities by default.
    pub grant_by_default: bool,
    /// List of explicitly granted capabilities.
    pub granted_capabilities: Vec<String>,
    /// List of explicitly denied capabilities.
    pub denied_capabilities: Vec<String>,
}
