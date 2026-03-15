//! Module registry implementation.

use crate::core::contracts::{ModuleInstallRecord, ModuleKind};
use std::collections::HashMap;

/// Registry for installed modules.
pub struct ModuleRegistry {
    modules: HashMap<String, ModuleInstallRecord>,
}

impl ModuleRegistry {
    /// Creates a new empty registry.
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
        }
    }

    /// Returns all installed modules.
    pub fn installed_modules(&self) -> Vec<&ModuleInstallRecord> {
        self.modules.values().collect()
    }

    /// Returns a module by ID.
    pub fn get_module(&self, id: &str) -> Option<&ModuleInstallRecord> {
        self.modules.get(id)
    }

    /// Registers a module.
    pub fn register(&mut self, record: ModuleInstallRecord) {
        self.modules.insert(record.id.clone(), record);
    }

    /// Unregisters a module.
    pub fn unregister(&mut self, id: &str) -> Option<ModuleInstallRecord> {
        self.modules.remove(id)
    }

    /// Returns modules by kind.
    pub fn modules_by_kind(&self, _kind: ModuleKind) -> Vec<&ModuleInstallRecord> {
        self.modules
            .values()
            .filter(|_m| true) // TODO: Add kind field to ModuleInstallRecord
            .collect()
    }
}

impl Default for ModuleRegistry {
    fn default() -> Self {
        Self::new()
    }
}
