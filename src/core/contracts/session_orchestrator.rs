//! Session orchestrator contract.

use super::turn_request::TurnRequest;
use super::turn_result::TurnResult;

/// Orchestrator for managing sessions.
pub trait SessionOrchestrator: Send + Sync {
    /// Submits a turn request.
    fn submit(&self, request: TurnRequest) -> anyhow::Result<TurnResult>;
}
