//! Observer contract - adapted from redclaw traits.

use serde::{Deserialize, Serialize};

/// Observer event.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ObserverEvent {
    /// Event name.
    pub name: String,
    /// Event data.
    pub data: serde_json::Value,
}

/// Observer metric.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ObserverMetric {
    /// Metric name.
    pub name: String,
    /// Metric value.
    pub value: f64,
}

/// Observer trait for observability.
pub trait Observer: Send + Sync {
    /// Records an event.
    fn record_event(&self, event: ObserverEvent);

    /// Records a metric.
    fn record_metric(&self, metric: ObserverMetric);
}
