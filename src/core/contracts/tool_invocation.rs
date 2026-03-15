//! Tool invocation contract.

use serde::{Deserialize, Serialize};

/// Request to invoke a tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolInvocation {
    /// Tool ID.
    pub tool_id: String,
    /// Tool arguments.
    pub arguments: serde_json::Value,
}

/// Result of a tool invocation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolInvocationResult {
    /// Tool ID.
    pub tool_id: String,
    /// Result content.
    pub result: String,
    /// Whether the invocation was successful.
    pub success: bool,
}
