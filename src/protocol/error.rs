//! Stable protocol error contract.

use serde::{Deserialize, Serialize};

/// Stable protocol error categories for public clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProtocolErrorCode {
    ValidationError,
    AuthError,
    PolicyDenied,
    RuntimeUnavailable,
    InternalError,
}

/// Public protocol error payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProtocolError {
    /// Stable error category.
    pub code: ProtocolErrorCode,
    /// Client-safe error message.
    pub message: String,
}

impl ProtocolError {
    /// Constructs a protocol error.
    pub fn new(code: ProtocolErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }
}
