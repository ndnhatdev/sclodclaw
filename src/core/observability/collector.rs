//! Trace collection.

use crate::core::observability::trace_context::TraceContext;

pub struct TraceCollector {
    traces: Vec<TraceContext>,
}

impl TraceCollector {
    pub fn new() -> Self {
        Self { traces: Vec::new() }
    }

    pub fn collect(&mut self, trace: TraceContext) {
        self.traces.push(trace);
    }
}

impl Default for TraceCollector {
    fn default() -> Self {
        Self::new()
    }
}
