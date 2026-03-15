//! Tool contract - executable capabilities.

use serde::{Deserialize, Serialize};

/// Specification for a tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolSpec {
    /// Tool name.
    pub name: String,
    /// Tool description.
    pub description: String,
    /// Parameters schema (JSON Schema).
    pub parameters: serde_json::Value,
}

/// Result of tool execution.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolResult {
    /// Tool name.
    pub tool_name: String,
    /// Result content.
    pub content: String,
    /// Whether execution was successful.
    pub success: bool,
}

/// Tool trait for executable capabilities.
pub trait Tool: Send + Sync {
    /// Returns the tool specification.
    fn spec(&self) -> ToolSpec;

    /// Executes the tool with the given input.
    fn execute(&self, input: &str) -> anyhow::Result<ToolResult>;
}
