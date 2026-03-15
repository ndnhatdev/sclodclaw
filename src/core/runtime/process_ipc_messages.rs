//! IPC message envelope types for process modules.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TraceContext {
    #[serde(default)]
    pub traceparent: String,
    #[serde(default)]
    pub tracestate: String,
}

impl TraceContext {
    pub fn from_env() -> Self {
        Self {
            traceparent: std::env::var("TRACEPARENT").unwrap_or_default(),
            tracestate: std::env::var("TRACESTATE").unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestEnvelope {
    #[serde(rename = "type")]
    pub envelope_type: String,
    pub id: String,
    pub method: String,
    pub payload: serde_json::Value,
}

impl RequestEnvelope {
    pub fn new(
        id: impl Into<String>,
        method: impl Into<String>,
        payload: serde_json::Value,
    ) -> Self {
        Self {
            envelope_type: "request".to_string(),
            id: id.into(),
            method: method.into(),
            payload,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponseEnvelope {
    #[serde(rename = "type")]
    pub envelope_type: String,
    pub id: String,
    pub ok: bool,
    #[serde(default)]
    pub payload: Option<serde_json::Value>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub details: Option<serde_json::Value>,
    #[serde(default)]
    pub retryable: Option<bool>,
    #[serde(default)]
    pub retry_after_ms: Option<u64>,
}

impl ResponseEnvelope {
    pub fn ok(id: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            envelope_type: "response".to_string(),
            id: id.into(),
            ok: true,
            payload: Some(payload),
            code: None,
            message: None,
            details: None,
            retryable: None,
            retry_after_ms: None,
        }
    }

    pub fn err(id: impl Into<String>, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            envelope_type: "response".to_string(),
            id: id.into(),
            ok: false,
            payload: None,
            code: Some(code.into()),
            message: Some(message.into()),
            details: None,
            retryable: None,
            retry_after_ms: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct NotificationEnvelope {
    #[serde(rename = "type")]
    pub envelope_type: String,
    pub method: String,
    pub payload: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ProcessIpcMessage {
    #[serde(rename = "request")]
    Request {
        id: String,
        method: String,
        payload: serde_json::Value,
    },
    #[serde(rename = "response")]
    Response {
        id: String,
        ok: bool,
        #[serde(default)]
        payload: Option<serde_json::Value>,
        #[serde(default)]
        code: Option<String>,
        #[serde(default)]
        message: Option<String>,
        #[serde(default)]
        details: Option<serde_json::Value>,
        #[serde(default)]
        retryable: Option<bool>,
        #[serde(default)]
        retry_after_ms: Option<u64>,
    },
    #[serde(rename = "notification")]
    Notification {
        method: String,
        payload: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RunnerHealth {
    pub status: String,
    pub last_error: Option<String>,
    pub last_started_at: Option<String>,
    pub last_completed_at: Option<String>,
}
