//! Safe-mode boot path and baseline modules.
//!
//! Safe mode boots with minimal required modules when full activation fails.

use crate::core::contracts::{ActivationResult, ExecutionPolicy, ModuleTrustTier};
use crate::core::lifecycle::ModuleHost;

/// Baseline modules required for safe mode operation.
/// These modules must always be available and activated first.
pub const SAFE_MODE_BASELINE_MODULES: &[&str] = &[
    "runtime-native", // Core runtime
    "channel-cli",    // CLI channel for user interaction
];

/// Safe mode configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeModeConfig {
    /// Whether safe mode is enabled.
    pub enabled: bool,
    /// Minimum modules required for safe mode.
    pub minimum_modules: Vec<String>,
}

impl Default for SafeModeConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            minimum_modules: SAFE_MODE_BASELINE_MODULES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }
}

/// Boots in safe mode with minimum required modules.
/// Returns ActivationResult for each baseline module.
pub fn boot_safe_mode(host: &mut ModuleHost) -> anyhow::Result<Vec<ActivationResult>> {
    tracing::info!(
        "Booting safe mode with baseline modules: {:?}",
        SAFE_MODE_BASELINE_MODULES
    );

    let results = host.activate_safe_mode_baseline(SAFE_MODE_BASELINE_MODULES)?;

    // 2. Baseline-only activation
    tracing::info!("Safe mode keeps baseline-only activation scope");

    // 3. Recovery surface signaling
    tracing::info!("Recovery surface remains available through baseline modules");

    // 4. Log safe mode boot
    tracing::info!("Safe mode boot complete with {} modules", results.len());

    Ok(results)
}

/// Returns the safe mode execution policy.
pub fn safe_mode_policy() -> ExecutionPolicy {
    ExecutionPolicy {
        safe_mode: true,
        deny_by_default: true,
        require_checksum: true,
        require_signature_for: vec![ModuleTrustTier::ThirdParty],
    }
}

/// Checks if a module is part of the safe mode baseline.
pub fn is_baseline_module(module_id: &str) -> bool {
    SAFE_MODE_BASELINE_MODULES.contains(&module_id)
}

/// Verifies all baseline modules are activated.
pub fn verify_baseline_modules(results: &[ActivationResult]) -> bool {
    SAFE_MODE_BASELINE_MODULES.iter().all(|module_id| {
        results
            .iter()
            .find(|result| result.module_id == *module_id)
            .map(|result| result.success)
            .unwrap_or(false)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::modules_lock::ModulesLock;
    use crate::core::contracts::ModuleInstallRecord;
    use crate::core::lifecycle::LoaderConfig;
    use crate::core::registry::ModuleRegistry;
    use std::fs;

    fn write_manifest(dir: &std::path::Path, manifest: &str) {
        fs::create_dir_all(dir).expect("create module dir");
        fs::write(dir.join("manifest.json"), manifest).expect("write manifest");
    }

    fn canonical_manifest(id: &str, kind: &str, deps: &[&str]) -> String {
        let dependencies = deps
            .iter()
            .map(|dep| format!("{{\"id\":\"{dep}\"}}"))
            .collect::<Vec<_>>()
            .join(",");
        format!(
            r#"{{
                "id": "{id}",
                "name": "{id}",
                "version": "0.1.0",
                "kind": "{kind}",
                "engine": {{ "redhorse": ">=0.1.0 <0.2.0" }},
                "artifact": {{ "kind": "bundled", "entry": "modules/{kind}s/{id}" }},
                "execution": {{ "mode": "in_process" }},
                "trust": {{ "required": "official" }},
                "capabilities": {{ "requested": [], "parameterized": [] }},
                "dependencies": [{dependencies}],
                "config": {{ "schema": {{"type": "object"}}, "defaultFragment": {{}} }},
                "activation": {{ "events": ["startup"], "safeModeEligible": true }},
                "install": {{ "source": "bundled" }}
            }}"#
        )
    }

    #[test]
    fn safe_mode_uses_real_host_path_and_baseline_assertions() {
        let temp = tempfile::tempdir().expect("tempdir");

        write_manifest(
            &temp.path().join("src/modules/runtimes/runtime-native"),
            &canonical_manifest("runtime-native", "runtime", &[]),
        );

        write_manifest(
            &temp.path().join("src/modules/channels/channel-cli"),
            &canonical_manifest("channel-cli", "channel", &["runtime-native"]),
        );

        let lock_path = temp.path().join("state/modules.lock");
        let mut lock = ModulesLock::new();
        lock.add_module(ModuleInstallRecord::bundled_v1(
            "runtime-native",
            "0.1.0",
            true,
        ));
        lock.add_module(ModuleInstallRecord::bundled_v1(
            "channel-cli",
            "0.1.0",
            true,
        ));
        lock.save_atomic(&lock_path).expect("save lock");

        let mut host = ModuleHost::new(
            ModuleRegistry::new(),
            LoaderConfig {
                search_paths: vec![temp.path().join("src/modules")],
                lock_path,
                validate: true,
            },
        );

        let results = boot_safe_mode(&mut host).expect("safe mode boot");
        assert!(verify_baseline_modules(&results));
        assert_eq!(results.len(), SAFE_MODE_BASELINE_MODULES.len());
        assert!(results
            .iter()
            .all(|r| r.phase == crate::core::contracts::ActivationPhase::Activated && r.success));
    }
}
