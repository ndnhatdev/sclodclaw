//! Public authentication context for protocol clients.

use serde::{Deserialize, Serialize};

/// Stable auth context carried by protocol-aware clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AuthContext {
    /// Bearer token used for authenticated requests.
    pub bearer_token: Option<String>,
    /// Whether the runtime currently requires pairing authentication.
    pub require_pairing: bool,
}

impl AuthContext {
    /// Creates an unauthenticated auth context.
    pub fn unauthenticated(require_pairing: bool) -> Self {
        Self {
            bearer_token: None,
            require_pairing,
        }
    }

    /// Creates an authenticated auth context from a bearer token.
    pub fn with_bearer_token(token: impl Into<String>, require_pairing: bool) -> Self {
        Self {
            bearer_token: Some(token.into()),
            require_pairing,
        }
    }

    /// Returns true when a non-empty bearer token is present.
    pub fn is_authenticated(&self) -> bool {
        self.bearer_token
            .as_deref()
            .map_or(false, |token| !token.is_empty())
    }
}
