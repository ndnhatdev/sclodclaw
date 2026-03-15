//! Structured logging.

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StructuredLog {
    pub level: String,
    pub message: String,
    pub timestamp: u64,
}

pub fn log_info(message: &str) -> StructuredLog {
    StructuredLog {
        level: "INFO".to_string(),
        message: message.to_string(),
        timestamp: 0,
    }
}
