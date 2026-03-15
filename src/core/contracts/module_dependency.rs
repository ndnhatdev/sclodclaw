//! Module dependency definition.

use serde::{Deserialize, Serialize};

/// A dependency on another module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModuleDependency {
    /// The ID of the module this depends on.
    #[serde(rename = "id")]
    pub module_id: String,
    /// Optional semver range for the dependency.
    #[serde(default)]
    pub version: Option<String>,
    /// Whether this dependency is optional.
    #[serde(default)]
    pub optional: bool,
}
