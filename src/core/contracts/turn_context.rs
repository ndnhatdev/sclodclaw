//! Turn context for orchestration.

use super::session_id::SessionId;
use serde::{Deserialize, Serialize};

/// Context for a single turn of interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnContext {
    /// The session this turn belongs to.
    pub session_id: SessionId,
    /// The channel for this turn.
    pub channel: String,
    /// Whether safe mode is enabled.
    pub safe_mode: bool,
}
