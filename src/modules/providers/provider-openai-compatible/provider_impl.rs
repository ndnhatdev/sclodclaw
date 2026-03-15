//! OpenAI-compatible provider implementation via process runner.

use crate::core::contracts::{ChatRequest, ChatResponse, Provider};
use crate::core::runtime::ProcessModuleRunner;
use serde_json::json;

const MODULE_ID: &str = "provider-openai-compatible";

pub struct OpenAiCompatibleProvider {
    runner: ProcessModuleRunner,
}

impl OpenAiCompatibleProvider {
    pub fn new(runner: ProcessModuleRunner) -> Self {
        Self { runner }
    }
}

impl Provider for OpenAiCompatibleProvider {
    fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse> {
        let input = json!({
            "model": request.model,
            "messages": request.messages,
            "temperature": request.temperature,
        });

        let response = self
            .runner
            .invoke("provider.request", "provider-chat", input)?;

        if !response.ok {
            anyhow::bail!(
                "process provider failed: {} {}",
                response
                    .code
                    .unwrap_or_else(|| "ipc.invalid_payload".to_string()),
                response
                    .message
                    .unwrap_or_else(|| "unknown process provider error".to_string())
            );
        }

        let payload = response.payload.unwrap_or_else(|| json!({}));
        let content = payload
            .get("content")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();

        let model = payload
            .get("model")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        Ok(ChatResponse { content, model })
    }

    fn name(&self) -> &str {
        MODULE_ID
    }
}

pub fn create_provider() -> anyhow::Result<Box<dyn Provider>> {
    let runner = ProcessModuleRunner::from_env(MODULE_ID)?;
    Ok(Box::new(OpenAiCompatibleProvider::new(runner)))
}
