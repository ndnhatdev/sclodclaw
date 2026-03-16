//! Modules lockfile - install state authority.
//!
//! This file uses atomic write semantics (temp file + rename) to ensure
//! the lockfile is never corrupted during writes.

use crate::core::contracts::ModuleInstallRecord;
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::{collections::HashSet, ffi::OsString};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModulesLock {
    #[serde(rename = "schemaVersion", alias = "schema_version")]
    pub schema_version: u32,
    pub modules: Vec<ModuleInstallRecord>,
}

impl ModulesLock {
    pub const CURRENT_SCHEMA: u32 = 1;

    pub fn new() -> Self {
        Self {
            schema_version: Self::CURRENT_SCHEMA,
            modules: Vec::new(),
        }
    }

    pub fn load(path: &Path) -> anyhow::Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }
        let content = std::fs::read_to_string(path)?;
        let lock: ModulesLock = serde_json::from_str(&content)?;
        if lock.schema_version != Self::CURRENT_SCHEMA {
            anyhow::bail!(
                "Unsupported modules.lock schema version {} (expected {})",
                lock.schema_version,
                Self::CURRENT_SCHEMA
            );
        }
        lock.validate()?;
        Ok(lock)
    }

    /// Saves the lockfile atomically using temp file + rename pattern.
    /// This ensures the lockfile is never corrupted during writes.
    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        self.save_atomic(path)
    }

    pub fn save_atomic(&self, path: &Path) -> anyhow::Result<()> {
        self.validate()?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;

        // Write to temp file first
        let temp_path = sibling_temp_path(path);
        std::fs::write(&temp_path, &content)?;

        let rename_result = std::fs::rename(&temp_path, path);
        if let Err(err) = rename_result {
            let _ = std::fs::remove_file(&temp_path);
            return Err(err.into());
        }

        // Sync directory to ensure rename is persisted
        #[cfg(unix)]
        {
            if let Some(parent) = path.parent() {
                let dir = std::fs::File::open(parent)?;
                dir.sync_all()?;
            }
        }

        Ok(())
    }

    pub fn add_module(&mut self, record: ModuleInstallRecord) {
        if let Some(existing) = self.modules.iter_mut().find(|m| m.id == record.id) {
            *existing = record;
            return;
        }
        self.modules.push(record);
    }

    pub fn remove_module(&mut self, id: &str) {
        self.modules.retain(|m| m.id != id);
    }

    pub fn get_module(&self, id: &str) -> Option<&ModuleInstallRecord> {
        self.modules.iter().find(|m| m.id == id)
    }

    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> anyhow::Result<()> {
        let record = self
            .modules
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| anyhow::anyhow!("module {} not found", id))?;
        record.enabled = enabled;
        Ok(())
    }

    pub fn validate(&self) -> anyhow::Result<()> {
        let mut seen = HashSet::new();
        for record in &self.modules {
            if !seen.insert(record.id.clone()) {
                anyhow::bail!("duplicate module id in modules.lock: {}", record.id);
            }
            record.validate().map_err(|err| {
                anyhow::anyhow!("invalid modules.lock record {}: {err}", record.id)
            })?;
        }
        Ok(())
    }
}

impl Default for ModulesLock {
    fn default() -> Self {
        Self::new()
    }
}

fn sibling_temp_path(path: &Path) -> std::path::PathBuf {
    let file_name = path
        .file_name()
        .map_or_else(|| OsString::from("modules.lock"), OsString::from);
    let mut temp_name = file_name;
    temp_name.push(".tmp");
    path.with_file_name(temp_name)
}
