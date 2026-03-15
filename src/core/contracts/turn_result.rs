//! Turn result for orchestration.

use super::session_id::SessionId;
use serde::{Deserialize, Serialize};

/// Result of a turn of interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnResult {
    /// The session this result belongs to.
    pub session_id: SessionId,
    /// The result content.
    pub content: String,
    /// Tool calls made during this turn.
    pub tool_calls: Vec<String>,
}
