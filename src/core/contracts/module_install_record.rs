//! Module install record.

use super::execution_mode::ExecutionMode;
use super::module_manifest::{
    is_valid_module_id, ArtifactKind, InstallSource, ManifestArtifact, ManifestInstall,
    ManifestTrust, ModuleManifest,
};
use super::module_trust_tier::ModuleTrustTier;
use super::quarantine_state::QuarantineState;
use semver::Version;
use serde::{Deserialize, Serialize};
use std::path::Path;

const ALLOWED_VERIFICATION_STATES: &[&str] = &["verified", "unverified", "failed"];
const ALLOWED_QUARANTINE_REASONS: &[&str] =
    &["verification_failed", "manifest_invalid", "policy_blocked"];

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstallArtifactRecord {
    pub kind: ArtifactKind,
    pub entry: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstallSourceRecord {
    pub source: InstallSource,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TrustRecord {
    pub tier: ModuleTrustTier,
    #[serde(rename = "verificationState", alias = "verification_state")]
    pub verification_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SourceRecord {
    pub uri: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecutionRecord {
    #[serde(rename = "resolvedMode", alias = "resolved_mode")]
    pub resolved_mode: ExecutionMode,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct QuarantineRecord {
    pub state: QuarantineState,
    pub reason: Option<String>,
    pub since: Option<String>,
}

/// Record of an installed module.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ModuleInstallRecord {
    /// Module ID.
    pub id: String,
    /// Installed version.
    pub version: String,
    /// Resolved artifact metadata.
    pub artifact: InstallArtifactRecord,
    /// Install source metadata.
    pub install: InstallSourceRecord,
    /// Trust and verification status.
    pub trust: TrustRecord,
    /// Artifact location metadata.
    pub source: SourceRecord,
    /// Effective host-resolved execution mode.
    pub execution: ExecutionRecord,
    /// Quarantine status information.
    pub quarantine: QuarantineRecord,
    /// Whether the module is enabled.
    pub enabled: bool,
    /// Verified checksum when available.
    pub checksum: Option<String>,
    /// Signature material reference when available.
    pub signature: Option<String>,
    /// Previous installed version when this is an upgrade.
    #[serde(rename = "previousVersion", alias = "previous_version")]
    pub previous_version: Option<String>,
    /// Previous checksum for upgrade tracking.
    #[serde(rename = "previousChecksum", alias = "previous_checksum")]
    pub previous_checksum: Option<String>,
}

impl From<ManifestArtifact> for InstallArtifactRecord {
    fn from(value: ManifestArtifact) -> Self {
        Self {
            kind: value.kind,
            entry: value.entry,
        }
    }
}

impl From<ManifestInstall> for InstallSourceRecord {
    fn from(value: ManifestInstall) -> Self {
        Self {
            source: value.source,
        }
    }
}

impl From<ManifestTrust> for TrustRecord {
    fn from(value: ManifestTrust) -> Self {
        let tier = match value.required {
            super::module_manifest::TrustRequirement::Official => ModuleTrustTier::Official,
            super::module_manifest::TrustRequirement::Reviewed => ModuleTrustTier::Reviewed,
            super::module_manifest::TrustRequirement::ThirdParty => ModuleTrustTier::ThirdParty,
        };
        Self {
            tier,
            verification_state: "verified".to_string(),
        }
    }
}

impl ModuleInstallRecord {
    /// Create a bundled module record (for bundled modules only).
    pub fn bundled_v1(id: impl Into<String>, version: impl Into<String>, enabled: bool) -> Self {
        let id = id.into();
        Self {
            id: id.clone(),
            version: version.into(),
            artifact: InstallArtifactRecord {
                kind: ArtifactKind::Bundled,
                entry: default_bundled_entry(&id),
            },
            install: InstallSourceRecord {
                source: InstallSource::Bundled,
            },
            trust: TrustRecord {
                tier: ModuleTrustTier::Official,
                verification_state: "verified".to_string(),
            },
            source: SourceRecord {
                uri: None,
                path: None,
            },
            execution: ExecutionRecord {
                resolved_mode: default_bundled_resolved_mode(&id),
            },
            quarantine: QuarantineRecord {
                state: QuarantineState::Clear,
                reason: None,
                since: None,
            },
            enabled,
            checksum: None,
            signature: None,
            previous_version: None,
            previous_checksum: None,
        }
    }

    /// Create an install record from manifest and source information (truthful for installed modules).
    pub fn from_manifest(
        manifest: &ModuleManifest,
        source: InstallSource,
        artifact_path: Option<&Path>,
        enabled: bool,
    ) -> Self {
        let (uri, path) = match &source {
            InstallSource::LocalDir => {
                (None, artifact_path.map(|p| p.to_string_lossy().to_string()))
            }
            InstallSource::Archive => {
                (None, artifact_path.map(|p| p.to_string_lossy().to_string()))
            }
            InstallSource::Bundled => (None, None),
            InstallSource::Registry => {
                (artifact_path.map(|p| p.to_string_lossy().to_string()), None)
            }
        };

        Self {
            id: manifest.id.clone(),
            version: manifest.version.clone(),
            artifact: InstallArtifactRecord {
                kind: manifest.artifact.kind.clone(),
                entry: manifest.artifact.entry.clone(),
            },
            install: InstallSourceRecord { source },
            trust: TrustRecord {
                tier: match manifest.trust.required {
                    crate::core::contracts::module_manifest::TrustRequirement::Official => {
                        ModuleTrustTier::Official
                    }
                    crate::core::contracts::module_manifest::TrustRequirement::Reviewed => {
                        ModuleTrustTier::Reviewed
                    }
                    crate::core::contracts::module_manifest::TrustRequirement::ThirdParty => {
                        ModuleTrustTier::ThirdParty
                    }
                },
                verification_state: "verified".to_string(),
            },
            source: SourceRecord { uri, path },
            execution: ExecutionRecord {
                resolved_mode: manifest.execution.mode,
            },
            quarantine: QuarantineRecord {
                state: QuarantineState::Clear,
                reason: None,
                since: None,
            },
            enabled,
            checksum: None,
            signature: None,
            previous_version: None,
            previous_checksum: None,
        }
    }

    pub fn with_quarantine_state(mut self, state: QuarantineState) -> Self {
        self.quarantine.state = state;
        match state {
            QuarantineState::Clear => {
                self.quarantine.reason = None;
                self.quarantine.since = None;
            }
            QuarantineState::Quarantined => {
                if self.quarantine.reason.is_none() {
                    self.quarantine.reason = Some("policy_blocked".to_string());
                }
            }
        }
        self
    }

    pub fn validate(&self) -> Result<(), String> {
        if !is_valid_module_id(&self.id) {
            return Err(format!("invalid module id: {}", self.id));
        }
        if Version::parse(&self.version).is_err() {
            return Err(format!("invalid module version: {}", self.version));
        }
        if self.artifact.entry.trim().is_empty() {
            return Err(format!("artifact.entry cannot be empty for {}", self.id));
        }
        if self.artifact.kind == ArtifactKind::Bundled
            && (!self.artifact.entry.starts_with("modules/")
                || self.artifact.entry.ends_with(".rs"))
        {
            return Err(format!(
                "bundled artifact.entry must be a canonical logical path for {}",
                self.id
            ));
        }
        if !ALLOWED_VERIFICATION_STATES.contains(&self.trust.verification_state.as_str()) {
            return Err(format!(
                "invalid trust.verificationState for {}: {}",
                self.id, self.trust.verification_state
            ));
        }
        if self.execution.resolved_mode == ExecutionMode::Wasm {
            return Err(format!(
                "resolved execution mode cannot be wasm in v1: {}",
                self.id
            ));
        }
        match self.install.source {
            InstallSource::Bundled => {}
            InstallSource::LocalDir => {
                if self
                    .source
                    .path
                    .as_ref()
                    .is_none_or(|path| path.trim().is_empty())
                {
                    return Err(format!(
                        "local_dir install source requires source.path: {}",
                        self.id
                    ));
                }
            }
            InstallSource::Archive | InstallSource::Registry => {
                let has_uri = self
                    .source
                    .uri
                    .as_ref()
                    .is_some_and(|uri| !uri.trim().is_empty());
                let has_path = self
                    .source
                    .path
                    .as_ref()
                    .is_some_and(|path| !path.trim().is_empty());
                if !has_uri && !has_path {
                    return Err(format!(
                        "archive/registry install source requires source.uri or source.path: {}",
                        self.id
                    ));
                }
            }
        }
        match self.quarantine.state {
            QuarantineState::Clear => {
                if self.quarantine.reason.is_some() || self.quarantine.since.is_some() {
                    return Err(format!(
                        "clear quarantine state must not carry reason/since: {}",
                        self.id
                    ));
                }
            }
            QuarantineState::Quarantined => {
                let reason =
                    self.quarantine.reason.as_deref().ok_or_else(|| {
                        format!("quarantined record requires reason: {}", self.id)
                    })?;
                if !ALLOWED_QUARANTINE_REASONS.contains(&reason) {
                    return Err(format!(
                        "invalid quarantine reason for {}: {}",
                        self.id, reason
                    ));
                }
            }
        }

        Ok(())
    }
}

fn default_bundled_entry(module_id: &str) -> String {
    let kind_segment = if module_id.starts_with("runtime-") {
        "runtimes"
    } else if module_id.starts_with("channel-") {
        "channels"
    } else if module_id.starts_with("provider-") {
        "providers"
    } else if module_id.starts_with("tool-") {
        "tools"
    } else if module_id.starts_with("memory-") {
        "memory"
    } else if module_id.starts_with("observer-") {
        "observers"
    } else {
        "modules"
    };

    format!("modules/{kind_segment}/{module_id}")
}

fn default_bundled_resolved_mode(module_id: &str) -> ExecutionMode {
    if module_id.starts_with("provider-") || module_id.starts_with("tool-") {
        ExecutionMode::Process
    } else {
        ExecutionMode::InProcess
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_v1_produces_a_valid_record() {
        let record = ModuleInstallRecord::bundled_v1("provider-openai-compatible", "0.1.0", true);
        assert_eq!(
            record.artifact.entry,
            "modules/providers/provider-openai-compatible"
        );
        assert_eq!(record.execution.resolved_mode, ExecutionMode::Process);
        record
            .validate()
            .expect("bundled_v1 record should validate");
    }
}
