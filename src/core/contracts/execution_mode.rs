//! Execution mode enumeration.

use serde::{Deserialize, Serialize};

/// How a module is executed.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionMode {
    /// In-process execution (shared memory space).
    InProcess,
    /// Process execution (isolated, via IPC).
    Process,
    /// Wasm execution (reserved in v1; parsed but rejected by validation).
    Wasm,
}
