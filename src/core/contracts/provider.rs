//! Provider contract - AI model inference backend.

use serde::{Deserialize, Serialize};

/// A chat message with role and content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    /// Role of the message sender (user, assistant, system).
    pub role: String,
    /// Message content.
    pub content: String,
}

/// Request for chat completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest {
    /// Model identifier.
    pub model: String,
    /// Messages in the conversation.
    pub messages: Vec<ChatMessage>,
    /// Temperature for sampling (0.0-2.0).
    pub temperature: f64,
}

/// Response from chat completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatResponse {
    /// Generated content.
    pub content: String,
    /// Model that generated the response.
    pub model: String,
}

/// Provider trait for AI model inference backends.
pub trait Provider: Send + Sync {
    /// Sends a chat request and returns the response.
    fn chat(&self, request: ChatRequest) -> anyhow::Result<ChatResponse>;

    /// Returns the provider name.
    fn name(&self) -> &str;
}
