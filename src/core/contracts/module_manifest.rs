//! Module manifest - canonical RedClaw module definition.

use super::execution_mode::ExecutionMode;
use super::module_dependency::ModuleDependency;
use super::module_kind::ModuleKind;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

const ALLOWED_CAPABILITIES: &[&str] = &[
    "events.publish",
    "session.read",
    "filesystem.workspace.read",
    "filesystem.workspace.write",
    "network.request",
    "process.spawn",
    "secrets.provider.openai",
    "secrets.provider.openrouter",
];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    Bundled,
    LocalDir,
    Archive,
    Process,
    Wasm,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TrustRequirement {
    Official,
    Reviewed,
    ThirdParty,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InstallSource {
    Bundled,
    LocalDir,
    Archive,
    Registry,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestEngine {
    pub redhorse: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestArtifact {
    pub kind: ArtifactKind,
    pub entry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestExecution {
    pub mode: ExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestTrust {
    pub required: TrustRequirement,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ParameterizedCapability {
    pub name: String,
    #[serde(default)]
    pub scope: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ManifestCapabilities {
    #[serde(default)]
    pub requested: Vec<String>,
    #[serde(default)]
    pub parameterized: Vec<ParameterizedCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ManifestConfig {
    pub schema: serde_json::Value,
    #[serde(rename = "defaultFragment")]
    pub default_fragment: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestActivation {
    pub events: Vec<String>,
    #[serde(rename = "safeModeEligible")]
    pub safe_mode_eligible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ManifestInstall {
    pub source: InstallSource,
}

/// Canonical manifest for a RedClaw module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ModuleManifest {
    /// Unique module identifier.
    pub id: String,
    /// Human-readable module name.
    pub name: String,
    /// Semantic version of the module.
    pub version: String,
    /// The kind of module.
    pub kind: ModuleKind,
    /// Compatibility engine requirements.
    pub engine: ManifestEngine,
    /// Artifact packaging metadata.
    pub artifact: ManifestArtifact,
    /// Requested execution semantics.
    pub execution: ManifestExecution,
    /// Required trust level for activation.
    pub trust: ManifestTrust,
    /// Capability declarations.
    pub capabilities: ManifestCapabilities,
    /// Dependencies on other modules.
    #[serde(default)]
    pub dependencies: Vec<ModuleDependency>,
    /// Module-owned config schema and defaults.
    pub config: ManifestConfig,
    /// Activation trigger metadata.
    pub activation: ManifestActivation,
    /// Install source metadata.
    pub install: ManifestInstall,
}

impl ModuleManifest {
    pub fn execution_mode(&self) -> ExecutionMode {
        self.execution.mode
    }

    /// Validates the manifest structure.
    pub fn validate(&self) -> Result<(), String> {
        if self.id.is_empty() {
            return Err("Module ID cannot be empty".to_string());
        }
        if !is_valid_module_id(&self.id) {
            return Err("Module ID must be kebab-case".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Module name cannot be empty".to_string());
        }
        if self.kind == ModuleKind::App {
            return Err("module kind `app` is not part of manifest v1".to_string());
        }
        if self.version.trim().is_empty() {
            return Err("Module version cannot be empty".to_string());
        }
        if Version::parse(&self.version).is_err() {
            return Err(format!("module version must be semver: {}", self.version));
        }
        if self.engine.redhorse.trim().is_empty() {
            return Err("engine.redhorse cannot be empty".to_string());
        }
        parse_version_req(&self.engine.redhorse).map_err(|_| {
            format!(
                "engine.redhorse must be a valid semver range: {}",
                self.engine.redhorse
            )
        })?;
        validate_artifact_entry(&self.artifact.kind, &self.artifact.entry)?;
        if !self.config.schema.is_object() {
            return Err("config.schema must be a JSON object".to_string());
        }
        if !self.config.default_fragment.is_object() {
            return Err("config.defaultFragment must be a JSON object".to_string());
        }
        if self.activation.events.is_empty() {
            return Err("activation.events must contain at least one event".to_string());
        }
        for event in &self.activation.events {
            if event.trim().is_empty() {
                return Err("activation.events cannot contain empty strings".to_string());
            }
        }

        let mut requested_seen = HashSet::new();
        for dep in &self.dependencies {
            if dep.module_id.is_empty() {
                return Err("dependency id cannot be empty".to_string());
            }
            if !is_valid_module_id(&dep.module_id) {
                return Err(format!(
                    "dependency id must be kebab-case: {}",
                    dep.module_id
                ));
            }
            if dep.module_id == self.id {
                return Err("module cannot depend on itself".to_string());
            }
            if let Some(version_req) = &dep.version {
                if version_req.trim().is_empty() {
                    return Err(format!(
                        "dependency version requirement cannot be empty: {}",
                        dep.module_id
                    ));
                }
                parse_version_req(version_req).map_err(|_| {
                    format!(
                        "dependency version must be a valid semver range for {}: {}",
                        dep.module_id, version_req
                    )
                })?;
            }
        }
        for capability in &self.capabilities.requested {
            if !ALLOWED_CAPABILITIES.contains(&capability.as_str()) {
                return Err(format!("unknown requested capability: {capability}"));
            }
            if !requested_seen.insert(capability.clone()) {
                return Err(format!("duplicate requested capability: {capability}"));
            }
        }
        let mut parameterized_seen = HashSet::new();
        for capability in &self.capabilities.parameterized {
            if !ALLOWED_CAPABILITIES.contains(&capability.name.as_str()) {
                return Err(format!(
                    "unknown parameterized capability: {}",
                    capability.name
                ));
            }
            if !capability.scope.is_object() {
                return Err(format!(
                    "parameterized capability scope must be an object: {}",
                    capability.name
                ));
            }
            if !parameterized_seen.insert(capability.name.clone()) {
                return Err(format!(
                    "duplicate parameterized capability: {}",
                    capability.name
                ));
            }
            if requested_seen.contains(&capability.name) {
                return Err(format!(
                    "capability declared in both requested and parameterized sets: {}",
                    capability.name
                ));
            }
        }
        if matches!(self.kind, ModuleKind::Provider | ModuleKind::Tool)
            && self.execution_mode() != ExecutionMode::Process
        {
            return Err("side-effectful module kinds must run in process mode".to_string());
        }
        match self.artifact.kind {
            ArtifactKind::Bundled => {}
            ArtifactKind::LocalDir | ArtifactKind::Archive => {
                if self.execution_mode() == ExecutionMode::InProcess {
                    return Err(
                        "local_dir/archive artifacts require process execution in v1".to_string(),
                    );
                }
            }
            ArtifactKind::Process => {
                if self.execution_mode() != ExecutionMode::Process {
                    return Err("process artifact kind requires process execution mode".to_string());
                }
            }
            ArtifactKind::Wasm => {
                return Err("wasm artifact/execution is reserved and rejected in v1".to_string());
            }
        }
        if self.execution_mode() == ExecutionMode::Wasm {
            return Err("wasm artifact/execution is reserved and rejected in v1".to_string());
        }
        Ok(())
    }
}

pub(crate) fn is_valid_module_id(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }

    let mut prev_dash = false;
    let mut all_valid = true;
    for ch in chars {
        if ch == '-' {
            if prev_dash {
                all_valid = false;
                break;
            }
            prev_dash = true;
            continue;
        }
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
            all_valid = false;
            break;
        }
        prev_dash = false;
    }

    all_valid && !value.ends_with('-')
}

fn validate_artifact_entry(kind: &ArtifactKind, entry: &str) -> Result<(), String> {
    if entry.trim().is_empty() {
        return Err("artifact.entry cannot be empty".to_string());
    }
    if entry.contains("..") || entry.contains('\\') || entry.starts_with('/') {
        return Err("artifact.entry must be a logical descriptor without path escapes".to_string());
    }
    if matches!(kind, ArtifactKind::Bundled) {
        if !entry.starts_with("modules/") {
            return Err("bundled artifact.entry must start with modules/".to_string());
        }
        if entry.ends_with(".rs") {
            return Err(
                "bundled artifact.entry must be a logical module path, not a source file"
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn parse_version_req(value: &str) -> Result<VersionReq, semver::Error> {
    VersionReq::parse(value).or_else(|_| VersionReq::parse(&normalize_version_req(value)))
}

fn normalize_version_req(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(", ")
}
