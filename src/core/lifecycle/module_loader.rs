//! Module loading patterns.

use crate::core::contracts::ModuleManifest;
use std::path::{Path, PathBuf};

pub fn load_manifest(manifest_path: &Path) -> anyhow::Result<ModuleManifest> {
    let content = std::fs::read_to_string(manifest_path)?;
    let manifest: ModuleManifest = serde_json::from_str(&content)?;
    Ok(manifest)
}

#[derive(Debug, Clone)]
pub struct LoaderConfig {
    pub search_paths: Vec<PathBuf>,
    pub lock_path: PathBuf,
    pub validate: bool,
}

impl Default for LoaderConfig {
    fn default() -> Self {
        Self {
            search_paths: vec![PathBuf::from("src/modules")],
            lock_path: PathBuf::from("state/modules.lock"),
            validate: true,
        }
    }
}
