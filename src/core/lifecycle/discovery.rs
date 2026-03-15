//! Module discovery.

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Discovered module candidate.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ModuleCandidate {
    pub manifest_path: PathBuf,
    pub module_dir: PathBuf,
    pub is_local: bool,
}

pub fn discover_modules(search_path: &Path) -> Vec<ModuleCandidate> {
    let mut candidates = Vec::new();
    if !search_path.exists() {
        return candidates;
    }

    let entries = match std::fs::read_dir(search_path) {
        Ok(entries) => entries,
        Err(err) => {
            tracing::warn!(
                "Failed to read module search path {}: {err}",
                search_path.display()
            );
            return candidates;
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }

        // Canonical nested form: <kind>/<module>/manifest.json
        let nested = discover_nested_candidates(&path);
        if !nested.is_empty() {
            candidates.extend(nested);
            continue;
        }

        // Flat legacy form: <module>/manifest.json
        let manifest_path = path.join("manifest.json");
        if manifest_path.exists() {
            candidates.push(ModuleCandidate {
                manifest_path,
                module_dir: path,
                is_local: true,
            });
        }
    }

    candidates
}

fn discover_nested_candidates(kind_dir: &Path) -> Vec<ModuleCandidate> {
    let mut candidates = Vec::new();
    let modules = match std::fs::read_dir(kind_dir) {
        Ok(entries) => entries,
        Err(_) => return candidates,
    };

    for module_entry in modules.flatten() {
        let module_dir = module_entry.path();
        if !module_dir.is_dir() {
            continue;
        }

        let manifest_path = module_dir.join("manifest.json");
        if manifest_path.exists() {
            candidates.push(ModuleCandidate {
                manifest_path,
                module_dir,
                is_local: true,
            });
        }
    }

    candidates
}
