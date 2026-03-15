//! Module capability definition.

use serde::{Deserialize, Serialize};

/// A capability that a module can have.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleCapability {
    /// Capability name.
    pub name: String,
    /// Capability version.
    pub version: String,
}
