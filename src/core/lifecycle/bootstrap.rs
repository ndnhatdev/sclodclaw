use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::config::state_paths_for;
use crate::core::config::modules_lock::ModulesLock;
use crate::core::contracts::{ActivationPhase, ModuleHealth};
use crate::core::lifecycle::SAFE_MODE_BASELINE_MODULES;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationBootstrap {
    pub config_dir: PathBuf,
    pub modules_lock_path: PathBuf,
    pub baseline_modules: Vec<String>,
}

impl ActivationBootstrap {
    pub fn from_config_dir(config_dir: &Path) -> Self {
        let paths = state_paths_for(config_dir);
        Self {
            config_dir: config_dir.to_path_buf(),
            modules_lock_path: paths.modules_lock_path,
            baseline_modules: SAFE_MODE_BASELINE_MODULES
                .iter()
                .map(|id| (*id).to_string())
                .collect(),
        }
    }

    pub async fn load_install_state(&self) -> anyhow::Result<Option<ModulesLock>> {
        if !self.modules_lock_path.exists() {
            return Ok(None);
        }

        let path = self.modules_lock_path.clone();
        let lock = tokio::task::spawn_blocking(move || ModulesLock::load(&path))
            .await
            .map_err(|err| anyhow::anyhow!("failed to join modules.lock load task: {err}"))??;

        Ok(Some(lock))
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SafeModeReason {
    ActivationFailure,
    InvalidModulesLock,
    VerificationFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LifecycleCheckpoint {
    pub phase: ActivationPhase,
    pub diagnostics: Vec<String>,
    pub health: Option<ModuleHealth>,
}
