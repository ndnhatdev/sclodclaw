//! Shell tool implementation via process runner.

use crate::core::contracts::{Tool, ToolResult, ToolSpec};
use crate::core::runtime::ProcessModuleRunner;
use serde_json::json;

const MODULE_ID: &str = "tool-shell";

pub struct ShellTool {
    runner: ProcessModuleRunner,
}

impl ShellTool {
    pub fn new(runner: ProcessModuleRunner) -> Self {
        Self { runner }
    }
}

impl Tool for ShellTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec {
            name: "shell".to_string(),
            description: "Execute shell commands through process runner boundary".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "timeout": {"type": "integer"}
                },
                "required": ["command"]
            }),
        }
    }

    fn execute(&self, input: &str) -> anyhow::Result<ToolResult> {
        let response =
            self.runner
                .invoke("tool.invoke", "tool-shell-exec", json!({"command": input}))?;

        if !response.ok {
            anyhow::bail!(
                "process tool failed: {} {}",
                response
                    .code
                    .unwrap_or_else(|| "ipc.invalid_payload".to_string()),
                response
                    .message
                    .unwrap_or_else(|| "unknown process tool error".to_string())
            );
        }

        let payload = response.payload.unwrap_or_else(|| json!({}));
        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        Ok(ToolResult {
            tool_name: "shell".to_string(),
            content,
            success: true,
        })
    }
}

pub fn create_tool() -> anyhow::Result<Box<dyn Tool>> {
    let runner = ProcessModuleRunner::from_env(MODULE_ID)?;
    Ok(Box::new(ShellTool::new(runner)))
}
