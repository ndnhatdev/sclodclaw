//! Message envelope for cross-module communication.

use super::session_id::SessionId;
use serde::{Deserialize, Serialize};

/// Envelope for messages between modules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MessageEnvelope {
    /// The session this message belongs to.
    pub session_id: SessionId,
    /// The channel this message came from.
    pub channel: String,
    /// The sender of this message.
    pub sender: String,
    /// The message content.
    pub content: String,
}
