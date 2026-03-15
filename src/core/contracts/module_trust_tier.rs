//! Module trust tier enumeration.

use serde::{Deserialize, Serialize};

/// Trust tier for a module, determining verification requirements.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleTrustTier {
    /// Official Redhorse modules (highest trust).
    Official,
    /// Reviewed modules (verified publisher tier in v1 docs).
    #[serde(alias = "first_party")]
    Reviewed,
    /// Third-party modules (community, requires verification).
    ThirdParty,
}
