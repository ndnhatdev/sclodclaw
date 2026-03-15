//! Module activation flow.

use crate::core::contracts::{ActivationPhase, ActivationResult, ExecutionMode, ModuleManifest};
use crate::core::runtime::ProcessModuleRunner;

pub fn activate_module(manifest: &ModuleManifest) -> ActivationResult {
    if manifest.execution_mode() == ExecutionMode::Process {
        let runner = match ProcessModuleRunner::from_env(&manifest.id) {
            Ok(runner) => runner,
            Err(err) => {
                return ActivationResult {
                    module_id: manifest.id.clone(),
                    phase: ActivationPhase::Activated,
                    success: false,
                    diagnostics: vec![format!("process runner unavailable: {err}")],
                };
            }
        };

        return match runner.activate() {
            Ok(response) if response.ok => ActivationResult {
                module_id: manifest.id.clone(),
                phase: ActivationPhase::Activated,
                success: true,
                diagnostics: vec!["Process module activated through runner".to_string()],
            },
            Ok(response) => ActivationResult {
                module_id: manifest.id.clone(),
                phase: ActivationPhase::Activated,
                success: false,
                diagnostics: vec![format!(
                    "process activation failed: {} {}",
                    response
                        .code
                        .unwrap_or_else(|| "ipc.invalid_payload".to_string()),
                    response
                        .message
                        .unwrap_or_else(|| "unknown runner error".to_string())
                )],
            },
            Err(err) => ActivationResult {
                module_id: manifest.id.clone(),
                phase: ActivationPhase::Activated,
                success: false,
                diagnostics: vec![format!("process activation error: {err}")],
            },
        };
    }

    ActivationResult {
        module_id: manifest.id.clone(),
        phase: ActivationPhase::Activated,
        success: true,
        diagnostics: vec!["Module activated".to_string()],
    }
}

pub fn deactivate_module(module_id: &str) -> ActivationResult {
    ActivationResult {
        module_id: module_id.to_string(),
        phase: ActivationPhase::Activated,
        success: true,
        diagnostics: vec!["Module deactivated".to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::contracts::{
        ArtifactKind, ExecutionMode, InstallSource, ManifestActivation, ManifestArtifact,
        ManifestCapabilities, ManifestConfig, ManifestEngine, ManifestExecution, ManifestInstall,
        ManifestTrust, ModuleKind, TrustRequirement,
    };

    fn canonical_manifest() -> ModuleManifest {
        ModuleManifest {
            id: "provider-openai-compatible".to_string(),
            name: "Provider OpenAI Compatible".to_string(),
            version: "0.1.0".to_string(),
            kind: ModuleKind::Provider,
            engine: ManifestEngine {
                redhorse: ">=0.1.0 <0.2.0".to_string(),
            },
            artifact: ManifestArtifact {
                kind: ArtifactKind::Bundled,
                entry: "modules/providers/provider-openai-compatible".to_string(),
            },
            execution: ManifestExecution {
                mode: ExecutionMode::Process,
            },
            trust: ManifestTrust {
                required: TrustRequirement::Official,
            },
            capabilities: ManifestCapabilities {
                requested: vec![],
                parameterized: vec![],
            },
            dependencies: vec![],
            config: ManifestConfig {
                schema: serde_json::json!({"type": "object"}),
                default_fragment: serde_json::json!({}),
            },
            activation: ManifestActivation {
                events: vec!["startup".to_string()],
                safe_mode_eligible: true,
            },
            install: ManifestInstall {
                source: InstallSource::Bundled,
            },
        }
    }

    #[test]
    fn process_mode_requires_real_runner_path() {
        let manifest = canonical_manifest();

        std::env::remove_var("REDHORSE_PROCESS_MODULE_BIN");

        let result = activate_module(&manifest);
        assert!(!result.success);
        assert!(result
            .diagnostics
            .iter()
            .any(|d| d.contains("process runner unavailable")));
    }
}
