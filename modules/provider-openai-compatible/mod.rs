//! OpenAI-compatible provider wrapper.
use redhorse_contracts::{Provider, ChatRequest, ChatResponse, ChatMessage};

pub struct OpenAiCompatibleProvider { api_key: String, base_url: String }
impl OpenAiCompatibleProvider {
    pub fn new(api_key: String, base_url: String) -> Self { Self { api_key, base_url } }
}
impl Provider for OpenAiCompatibleProvider {
    fn chat(&self, _request: ChatRequest) -> anyhow::Result<ChatResponse> {
        Ok(ChatResponse { content: "Response".to_string(), model: "openai".to_string() })
    }
}
pub fn create_provider(api_key: String, base_url: String) -> Box<dyn Provider> {
    Box::new(OpenAiCompatibleProvider::new(api_key, base_url))
}
