//! Minimal stable client SDK helpers built on the public protocol seam.

use crate::protocol::{
    ClientCommand, ProtocolVersion, EVENTS_STREAM_PATH, PAIR_PATH, PUBLIC_HEALTH_PATH, WS_CHAT_PATH,
};
use urlencoding::encode;

/// Minimal URL and command helpers for app/client SDK consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientSdk {
    base_http_url: String,
}

impl ClientSdk {
    /// Creates a client SDK helper from an HTTP(S) base URL.
    pub fn new(base_http_url: impl Into<String>) -> Self {
        let base_http_url = normalize_base_http_url(base_http_url.into());
        Self { base_http_url }
    }

    /// Returns the normalized base HTTP URL.
    pub fn base_http_url(&self) -> &str {
        &self.base_http_url
    }

    /// Returns the public health endpoint URL.
    pub fn public_health_url(&self) -> String {
        format!("{}{}", self.base_http_url, PUBLIC_HEALTH_PATH)
    }

    /// Returns the pairing endpoint URL.
    pub fn pair_url(&self) -> String {
        format!("{}{}", self.base_http_url, PAIR_PATH)
    }

    /// Returns the SSE event stream endpoint URL.
    pub fn events_stream_url(&self) -> String {
        format!("{}{}", self.base_http_url, EVENTS_STREAM_PATH)
    }

    /// Returns the WebSocket sub-protocol token expected by the gateway.
    pub fn ws_subprotocol(&self) -> &'static str {
        ProtocolVersion::default().ws_subprotocol()
    }

    /// Returns a WebSocket URL for the chat endpoint.
    pub fn ws_chat_url(&self, token: Option<&str>, session_id: Option<&str>) -> String {
        let mut query = Vec::new();

        if let Some(token) = token {
            if !token.is_empty() {
                query.push(format!("token={}", encode(token)));
            }
        }

        if let Some(session_id) = session_id {
            if !session_id.is_empty() {
                query.push(format!("session_id={}", encode(session_id)));
            }
        }

        let base = http_to_ws_base(&self.base_http_url);
        if query.is_empty() {
            format!("{}{}", base, WS_CHAT_PATH)
        } else {
            format!("{}{}?{}", base, WS_CHAT_PATH, query.join("&"))
        }
    }

    /// Builds a backward-compatible message command payload.
    pub fn message_command(&self, content: impl Into<String>) -> ClientCommand {
        ClientCommand::message(content)
    }
}

fn normalize_base_http_url(value: String) -> String {
    let trimmed = value.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        "http://localhost:5555".to_string()
    } else {
        trimmed.to_string()
    }
}

fn http_to_ws_base(value: &str) -> String {
    if let Some(suffix) = value.strip_prefix("https://") {
        format!("wss://{suffix}")
    } else if let Some(suffix) = value.strip_prefix("http://") {
        format!("ws://{suffix}")
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::ClientSdk;

    #[test]
    fn ws_chat_url_carries_query_parameters() {
        let sdk = ClientSdk::new("http://localhost:5555/");
        assert_eq!(
            sdk.ws_chat_url(Some("tok"), Some("session-1")),
            "ws://localhost:5555/ws/chat?token=tok&session_id=session-1"
        );
    }

    #[test]
    fn ws_chat_url_handles_https_base() {
        let sdk = ClientSdk::new("https://example.com");
        assert_eq!(sdk.ws_chat_url(None, None), "wss://example.com/ws/chat");
    }

    #[test]
    fn empty_base_uses_default_gateway_origin() {
        let sdk = ClientSdk::new("   ");
        assert_eq!(sdk.base_http_url(), "http://localhost:5555");
    }
}
