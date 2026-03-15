//! Public client command contract.

use super::ProtocolSessionHandle;
use serde::{Deserialize, Serialize};

/// Commands accepted from app and SDK clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientCommand {
    /// Create a new protocol session.
    CreateSession,
    /// Resume an existing protocol session.
    ResumeSession { handle: ProtocolSessionHandle },
    /// Close an existing protocol session.
    CloseSession { handle: ProtocolSessionHandle },
    /// Submit a turn in a session-oriented flow.
    SubmitTurn {
        content: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        handle: Option<ProtocolSessionHandle>,
    },
    /// Backward-compatible single-turn message command used by existing web clients.
    #[serde(rename = "message")]
    Message { content: String },
}

impl ClientCommand {
    /// Builds a backward-compatible message command.
    pub fn message(content: impl Into<String>) -> Self {
        Self::Message {
            content: content.into(),
        }
    }

    /// Returns command content when the command carries turn text.
    pub fn content(&self) -> Option<&str> {
        match self {
            Self::SubmitTurn { content, .. } | Self::Message { content } => Some(content),
            _ => None,
        }
    }
}
