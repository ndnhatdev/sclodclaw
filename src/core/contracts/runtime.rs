//! Runtime contract - adapted from redclaw traits.

use serde::{Deserialize, Serialize};

/// Runtime adapter trait for module execution.
pub trait RuntimeAdapter: Send + Sync {
    /// Initializes the runtime.
    fn init(&self) -> anyhow::Result<()>;

    /// Runs a module.
    fn run(&self, module_id: &str) -> anyhow::Result<()>;

    /// Shuts down the runtime.
    fn shutdown(&self) -> anyhow::Result<()>;
}

/// Runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeConfig {
    /// Whether safe mode is enabled.
    pub safe_mode: bool,
    /// Maximum concurrent modules.
    pub max_concurrent_modules: usize,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            safe_mode: true,
            max_concurrent_modules: 10,
        }
    }
}
