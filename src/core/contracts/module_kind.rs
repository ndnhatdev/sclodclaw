//! Module kind enumeration - defines slot types in Redhorse module system.

use serde::{Deserialize, Serialize};

/// The kind of module, determining its slot and capabilities.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    /// Runtime module (core execution engine)
    Runtime,
    /// Provider module (AI model inference backends)
    Provider,
    /// Channel module (communication surfaces)
    Channel,
    /// Tool module (executable capabilities)
    Tool,
    /// Memory module (storage and retrieval)
    Memory,
    /// Observer module (observability and metrics)
    Observer,
    /// App module (product surfaces)
    App,
}

impl ModuleKind {
    /// Returns true if this module kind can be executed in-process.
    pub fn can_be_in_process(self) -> bool {
        matches!(
            self,
            ModuleKind::Runtime | ModuleKind::Memory | ModuleKind::Observer
        )
    }

    /// Returns true if this module kind should prefer process isolation.
    pub fn should_be_process_isolated(self) -> bool {
        matches!(
            self,
            ModuleKind::Provider | ModuleKind::Tool | ModuleKind::Channel | ModuleKind::App
        )
    }
}
