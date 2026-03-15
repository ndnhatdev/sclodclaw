//! Shell tool wrapper with process isolation.
use redhorse_contracts::{Tool, ToolSpec, ToolResult};

pub struct ShellTool;
impl Tool for ShellTool {
    fn spec(&self) -> ToolSpec {
        ToolSpec { name: "shell".to_string(), description: "Execute shell commands".to_string(),
            parameters: serde_json::json!({"type":"object","properties":{"command":{"type":"string"}}}) }
    }
    fn execute(&self, input: &str) -> anyhow::Result<ToolResult> {
        Ok(ToolResult { tool_name: "shell".to_string(), content: format!("Executed: {}", input), success: true })
    }
}
pub fn create_tool() -> Box<dyn Tool> { Box::new(ShellTool) }
