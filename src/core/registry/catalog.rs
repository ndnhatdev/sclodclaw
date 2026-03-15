//! Module catalog data.

use serde::{Deserialize, Serialize};

/// Catalog entry for a discoverable module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CatalogEntry {
    /// Module ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Module kind.
    pub kind: String,
    /// Publisher.
    pub publisher: String,
    /// Version.
    pub version: String,
}

/// Module catalog.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleCatalog {
    entries: Vec<CatalogEntry>,
}

impl ModuleCatalog {
    /// Creates a new catalog.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Adds an entry to the catalog.
    pub fn add(&mut self, entry: CatalogEntry) {
        self.entries.push(entry);
    }

    /// Returns all entries.
    pub fn entries(&self) -> &[CatalogEntry] {
        &self.entries
    }

    /// Finds an entry by ID.
    pub fn find(&self, id: &str) -> Option<&CatalogEntry> {
        self.entries.iter().find(|e| e.id == id)
    }
}
