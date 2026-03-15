//! Module artifact definition.

use serde::{Deserialize, Serialize};

/// An artifact produced by a module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleArtifact {
    /// The kind of artifact.
    pub kind: String,
    /// The entry point for this artifact.
    pub entry: String,
}
