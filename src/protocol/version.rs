//! Protocol version contract.

use serde::{Deserialize, Serialize};

/// Public protocol versions supported by the app/client seam.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ProtocolVersion {
    /// RedClaw public protocol v1.
    #[serde(rename = "redclaw.v1")]
    RedclawV1,
}

impl Default for ProtocolVersion {
    fn default() -> Self {
        Self::RedclawV1
    }
}

impl ProtocolVersion {
    /// Returns the WebSocket sub-protocol token for the version.
    pub const fn ws_subprotocol(self) -> &'static str {
        match self {
            Self::RedclawV1 => WS_SUBPROTOCOL_V1,
        }
    }
}

/// WebSocket sub-protocol token for v1 clients.
pub const WS_SUBPROTOCOL_V1: &str = "redclaw.v1";
