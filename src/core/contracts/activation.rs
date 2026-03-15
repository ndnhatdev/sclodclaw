//! Module activation contract.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActivationPhase {
    Discovered,
    ManifestLoaded,
    ConfigResolved,
    InstallStateResolved,
    DependencyValidated,
    SecurityValidated,
    RuntimeRegistered,
    Activated,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationRequest {
    pub module_id: String,
    pub config_fragment: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActivationResult {
    pub module_id: String,
    pub phase: ActivationPhase,
    pub success: bool,
    pub diagnostics: Vec<String>,
}
