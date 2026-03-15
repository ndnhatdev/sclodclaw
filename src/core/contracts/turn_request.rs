//! Turn request for orchestration.

use super::message_envelope::MessageEnvelope;
use super::turn_context::TurnContext;
use serde::{Deserialize, Serialize};

/// Request for a turn of interaction.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TurnRequest {
    /// The message envelope.
    pub envelope: MessageEnvelope,
    /// The turn context.
    pub context: TurnContext,
}
