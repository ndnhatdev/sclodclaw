//! State path resolution utilities.
//!
//! # Default Directory Policy (RedClaw-Native)
//!
//! - Primary default: `~/.redclaw` (canonical, `HOME_DIR`)
//! - NO legacy fallback (legacy paths are IGNORED)
//!
//! Legacy home directory usage is NOT detected or warned.
//! Users must explicitly migrate to ~/.redclaw.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use directories::UserDirs;

use crate::branding::{
    ACTIVE_WORKSPACE_STATE_FILE_NAME, CONFIG_FILE_NAME, HOME_DIR, MODULES_LOCK_FILE_NAME,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StatePaths {
    pub config_dir: PathBuf,
    pub workspace_dir: PathBuf,
    pub modules_lock_path: PathBuf,
    pub active_workspace_marker: PathBuf,
}

/// Returns the canonical config directory: `~/.redclaw`.
///
/// This is the ONLY default for all installations.
/// NO legacy fallback is provided.
pub fn default_config_dir() -> Result<PathBuf> {
    let home = UserDirs::new()
        .map(|u| u.home_dir().to_path_buf())
        .context("Could not find home directory")?;
    Ok(home.join(HOME_DIR))
}

pub fn default_config_and_workspace_dirs() -> Result<(PathBuf, PathBuf)> {
    let config_dir = default_config_dir()?;
    Ok((
        config_dir.clone(),
        resolve_workspace_dir_from_config_dir(&config_dir),
    ))
}

pub fn resolve_workspace_dir_from_config_dir(config_dir: &Path) -> PathBuf {
    config_dir.join("workspace")
}

pub fn active_workspace_state_path(default_dir: &Path) -> PathBuf {
    default_dir.join(ACTIVE_WORKSPACE_STATE_FILE_NAME)
}

pub fn modules_lock_path(config_dir: &Path) -> PathBuf {
    config_dir.join(MODULES_LOCK_FILE_NAME)
}

pub fn state_paths_for(config_dir: &Path) -> StatePaths {
    StatePaths {
        config_dir: config_dir.to_path_buf(),
        workspace_dir: resolve_workspace_dir_from_config_dir(config_dir),
        modules_lock_path: modules_lock_path(config_dir),
        active_workspace_marker: active_workspace_state_path(config_dir),
    }
}

pub fn resolve_config_dir_for_workspace(workspace_dir: &Path) -> (PathBuf, PathBuf) {
    let workspace_config_dir = workspace_dir.to_path_buf();
    if workspace_config_dir.join(CONFIG_FILE_NAME).exists() {
        return (
            workspace_config_dir.clone(),
            resolve_workspace_dir_from_config_dir(&workspace_config_dir),
        );
    }

    if workspace_dir
        .file_name()
        .is_some_and(|name| name == std::ffi::OsStr::new("workspace"))
    {
        if let Some(parent) = workspace_dir.parent() {
            let config_dir = parent.join(HOME_DIR);
            if config_dir.join(CONFIG_FILE_NAME).exists() {
                return (config_dir, workspace_config_dir);
            }
            return (config_dir, workspace_config_dir);
        }
    }

    (
        workspace_config_dir.clone(),
        resolve_workspace_dir_from_config_dir(&workspace_config_dir),
    )
}
