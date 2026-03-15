//! Execution policy for modules.

use super::module_trust_tier::ModuleTrustTier;
use serde::{Deserialize, Serialize};

/// Policy governing module execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExecutionPolicy {
    /// Whether safe mode is enabled.
    pub safe_mode: bool,
    /// Whether to deny by default.
    pub deny_by_default: bool,
    /// Whether checksums are required.
    pub require_checksum: bool,
    /// Which trust tiers require signatures.
    pub require_signature_for: Vec<ModuleTrustTier>,
}

impl Default for ExecutionPolicy {
    fn default() -> Self {
        Self {
            safe_mode: true,
            deny_by_default: true,
            require_checksum: true,
            require_signature_for: vec![ModuleTrustTier::ThirdParty],
        }
    }
}
