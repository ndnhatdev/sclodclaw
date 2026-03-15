//! Trace context propagation.

#[derive(Debug, Clone)]
pub struct TraceContext {
    pub trace_id: String,
    pub span_id: String,
}

impl TraceContext {
    pub fn new() -> Self {
        Self {
            trace_id: uuid::Uuid::new_v4().to_string(),
            span_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}
