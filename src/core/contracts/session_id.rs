//! Session ID type.

use serde::{Deserialize, Serialize};

/// A unique session identifier.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SessionId(pub String);

impl SessionId {
    /// Creates a new session ID from a string.
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Returns the session ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
