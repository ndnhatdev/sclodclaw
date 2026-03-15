//! Public client event contract.

use super::ProtocolErrorCode;
use serde::{Deserialize, Serialize};

/// Events emitted toward app and SDK clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientEvent {
    /// Runtime accepted a new turn.
    TurnUpdate { content: String },
    /// Runtime emitted a tool invocation summary.
    ToolInvocationSummary {
        name: String,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        duration_ms: Option<u128>,
    },
    /// Runtime completed a turn.
    Completion { full_response: String },
    /// Runtime failed a turn.
    Failure {
        code: ProtocolErrorCode,
        message: String,
    },
    /// Compatibility event used by existing dashboard consumers.
    #[serde(rename = "done")]
    Done { full_response: String },
    /// Compatibility event used by existing dashboard consumers.
    #[serde(rename = "error")]
    Error {
        message: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        code: Option<ProtocolErrorCode>,
        #[serde(skip_serializing_if = "Option::is_none")]
        component: Option<String>,
    },
}
