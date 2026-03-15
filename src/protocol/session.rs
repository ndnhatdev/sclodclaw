//! Session handle contract for app-facing protocol use.

use super::ProtocolVersion;
use serde::{Deserialize, Serialize};

/// Stable protocol session handle for create/resume/close flows.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ProtocolSessionHandle {
    /// Public session identifier.
    pub session_id: String,
    /// Protocol version associated with the session handle.
    #[serde(default)]
    pub version: ProtocolVersion,
}

impl ProtocolSessionHandle {
    /// Creates a v1 protocol session handle from a session id.
    pub fn new(session_id: impl Into<String>) -> Self {
        Self {
            session_id: session_id.into(),
            version: ProtocolVersion::default(),
        }
    }
}
