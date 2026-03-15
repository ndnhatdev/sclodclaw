//! Module host implementation.

use crate::core::config::modules_lock::ModulesLock;
use crate::core::contracts::{
    ActivationPhase, ActivationResult, ModuleInstallRecord, ModuleManifest, QuarantineState,
};
use crate::core::lifecycle::{
    activate_module, discover_modules, load_manifest, validate_manifest, DependencyGraph,
    LoaderConfig,
};
use crate::core::registry::ModuleRegistry;
use std::collections::HashMap;

pub struct ModuleHost {
    registry: ModuleRegistry,
    loader_config: LoaderConfig,
    manifests: HashMap<String, ModuleManifest>,
    activation_results: Vec<ActivationResult>,
}

impl ModuleHost {
    pub fn new(registry: ModuleRegistry, config: LoaderConfig) -> Self {
        Self {
            registry,
            loader_config: config,
            manifests: HashMap::new(),
            activation_results: Vec::new(),
        }
    }

    pub fn start(&mut self) -> anyhow::Result<()> {
        let lock = self.prepare_host_state()?;
        let activation_order = self.resolve_activation_order()?;

        self.activation_results.clear();
        for module_id in activation_order {
            let result = self.activate_with_lock(&module_id, &lock);
            self.activation_results.push(result);
        }

        Ok(())
    }

    pub fn stop(&mut self) -> anyhow::Result<()> {
        self.activation_results.clear();
        Ok(())
    }

    pub fn registry(&self) -> &ModuleRegistry {
        &self.registry
    }

    pub fn activation_results(&self) -> &[ActivationResult] {
        &self.activation_results
    }

    pub fn activate_safe_mode_baseline(
        &mut self,
        baseline_modules: &[&str],
    ) -> anyhow::Result<Vec<ActivationResult>> {
        let lock = self.prepare_host_state()?;
        self.activation_results.clear();
        let mut results = Vec::new();

        for module_id in baseline_modules {
            if !self.manifests.contains_key(*module_id) {
                results.push(failed_result(
                    module_id,
                    ActivationPhase::Discovered,
                    "baseline module not discovered",
                ));
                if let Some(last) = results.last() {
                    self.activation_results.push(last.clone());
                }
                continue;
            }

            let result = self.activate_with_lock(module_id, &lock);
            self.activation_results.push(result.clone());
            results.push(result);
        }

        Ok(results)
    }

    /// Registers a module with the host.
    pub fn register_module(&mut self, id: &str, _module: impl std::any::Any) -> anyhow::Result<()> {
        if self.registry.get_module(id).is_some() {
            anyhow::bail!("module already registered: {id}");
        }

        let manifest = self
            .manifests
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("manifest not loaded for module: {id}"))?;

        // For bundled modules loaded from src/modules/*/*, use bundled_v1
        let record = ModuleInstallRecord::bundled_v1(&manifest.id, &manifest.version, true);
        self.registry.register(record);
        Ok(())
    }

    fn prepare_host_state(&mut self) -> anyhow::Result<ModulesLock> {
        self.manifests.clear();

        let mut lock = ModulesLock::load(&self.loader_config.lock_path)?;
        let mut lock_dirty = false;

        for search_path in &self.loader_config.search_paths {
            for candidate in discover_modules(search_path) {
                let manifest = match load_manifest(&candidate.manifest_path) {
                    Ok(m) => m,
                    Err(err) => {
                        tracing::warn!(
                            "Skipping module candidate at {}: {err}",
                            candidate.manifest_path.display()
                        );
                        continue;
                    }
                };

                if self.loader_config.validate {
                    let validation = validate_manifest(&manifest);
                    if !validation.valid {
                        tracing::warn!(
                            "Skipping invalid module manifest {}: {}",
                            manifest.id,
                            validation.errors.join("; ")
                        );
                        continue;
                    }
                }

                if lock.get_module(&manifest.id).is_none() {
                    // For bundled modules loaded from src/modules/*/*, use bundled_v1
                    lock.add_module(ModuleInstallRecord::bundled_v1(
                        &manifest.id,
                        &manifest.version,
                        default_enabled_for_new_module(&manifest.id),
                    ));
                    lock_dirty = true;
                }

                self.manifests.insert(manifest.id.clone(), manifest);
            }
        }

        if lock_dirty {
            lock.save_atomic(&self.loader_config.lock_path)?;
        }

        Ok(lock)
    }

    fn resolve_activation_order(&self) -> anyhow::Result<Vec<String>> {
        let mut graph = DependencyGraph::new();
        for manifest in self.manifests.values() {
            let required_dependencies = manifest
                .dependencies
                .iter()
                .filter(|dep| !dep.optional)
                .map(|dep| dep.module_id.clone())
                .collect::<Vec<_>>();
            graph.add_module(&manifest.id, required_dependencies);
        }

        graph
            .validate_dag()
            .map_err(|err| anyhow::anyhow!("dependency validation failed: {err}"))?;

        graph
            .activation_order()
            .map_err(|err| anyhow::anyhow!("dependency ordering failed: {err}"))
    }

    fn activate_with_lock(&mut self, module_id: &str, lock: &ModulesLock) -> ActivationResult {
        let mut diagnostics = vec![];
        diagnostics.push("discovered".to_string());

        let Some(manifest) = self.manifests.get(module_id) else {
            return failed_result(
                module_id,
                ActivationPhase::Discovered,
                "module not discovered",
            );
        };

        diagnostics.push("manifest_loaded".to_string());
        diagnostics.push("config_resolved".to_string());

        let Some(lock_record) = lock.get_module(module_id) else {
            return failed_result(
                module_id,
                ActivationPhase::InstallStateResolved,
                "module missing from modules.lock",
            );
        };

        if !lock_record.enabled {
            return failed_result(
                module_id,
                ActivationPhase::InstallStateResolved,
                "module disabled in modules.lock",
            );
        }

        diagnostics.push("install_state_resolved".to_string());

        for dep in &manifest.dependencies {
            if dep.optional {
                continue;
            }

            if !self.manifests.contains_key(&dep.module_id) {
                return failed_result(
                    module_id,
                    ActivationPhase::DependencyValidated,
                    &format!("missing dependency: {}", dep.module_id),
                );
            }

            let Some(dep_record) = lock.get_module(&dep.module_id) else {
                return failed_result(
                    module_id,
                    ActivationPhase::DependencyValidated,
                    &format!("dependency missing from modules.lock: {}", dep.module_id),
                );
            };

            if !dep_record.enabled {
                return failed_result(
                    module_id,
                    ActivationPhase::DependencyValidated,
                    &format!("dependency disabled in modules.lock: {}", dep.module_id),
                );
            }

            if dep_record.quarantine.state != QuarantineState::Clear {
                return failed_result(
                    module_id,
                    ActivationPhase::DependencyValidated,
                    &format!("dependency quarantined in modules.lock: {}", dep.module_id),
                );
            }

            let dependency_active = self
                .activation_results
                .iter()
                .any(|result| result.module_id == dep.module_id && result.success);
            if !dependency_active {
                return failed_result(
                    module_id,
                    ActivationPhase::DependencyValidated,
                    &format!("dependency not activated successfully: {}", dep.module_id),
                );
            }
        }
        diagnostics.push("dependency_validated".to_string());

        if lock_record.quarantine.state != QuarantineState::Clear {
            return failed_result(
                module_id,
                ActivationPhase::SecurityValidated,
                "module quarantined in modules.lock",
            );
        }
        diagnostics.push("security_validated".to_string());

        self.registry.register(ModuleInstallRecord {
            id: lock_record.id.clone(),
            version: lock_record.version.clone(),
            artifact: lock_record.artifact.clone(),
            install: lock_record.install.clone(),
            trust: lock_record.trust.clone(),
            source: lock_record.source.clone(),
            execution: lock_record.execution.clone(),
            quarantine: lock_record.quarantine.clone(),
            enabled: lock_record.enabled,
            checksum: lock_record.checksum.clone(),
            signature: lock_record.signature.clone(),
            previous_version: lock_record.previous_version.clone(),
            previous_checksum: lock_record.previous_checksum.clone(),
        });
        diagnostics.push("runtime_registered".to_string());

        let activation = activate_module(manifest);
        diagnostics.extend(activation.diagnostics);

        ActivationResult {
            module_id: module_id.to_string(),
            phase: activation.phase,
            success: activation.success,
            diagnostics,
        }
    }
}

fn default_enabled_for_new_module(module_id: &str) -> bool {
    matches!(module_id, "runtime-native" | "channel-cli")
}

fn failed_result(module_id: &str, phase: ActivationPhase, reason: &str) -> ActivationResult {
    ActivationResult {
        module_id: module_id.to_string(),
        phase,
        success: false,
        diagnostics: vec![reason.to_string()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn write_manifest(dir: &std::path::Path, manifest: &str) {
        fs::create_dir_all(dir).expect("create module dir");
        fs::write(dir.join("manifest.json"), manifest).expect("write manifest");
    }

    fn canonical_manifest(id: &str, kind: &str, mode: &str, deps: &[&str]) -> String {
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
                "execution": {{ "mode": "{mode}" }},
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
    fn host_uses_modules_lock_for_enabled_state() {
        let temp = tempfile::tempdir().expect("tempdir");
        let modules_root = temp.path().join("src/modules/runtimes/runtime-native");
        write_manifest(
            &modules_root,
            &canonical_manifest("runtime-native", "runtime", "in_process", &[]),
        );

        let lock_path = temp.path().join("state/modules.lock");
        let mut lock = ModulesLock::new();
        lock.add_module(ModuleInstallRecord::bundled_v1(
            "runtime-native",
            "0.1.0",
            false,
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

        host.start().expect("host start");
        let result = host
            .activation_results()
            .iter()
            .find(|r| r.module_id == "runtime-native")
            .expect("runtime-native result");

        assert!(!result.success);
        assert_eq!(result.phase, ActivationPhase::InstallStateResolved);
    }

    #[test]
    fn host_activates_module_with_ordered_diagnostics() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_dir = temp.path().join("src/modules/runtimes/runtime-native");
        let channel_dir = temp.path().join("src/modules/channels/channel-cli");

        write_manifest(
            &runtime_dir,
            &canonical_manifest("runtime-native", "runtime", "in_process", &[]),
        );

        write_manifest(
            &channel_dir,
            &canonical_manifest("channel-cli", "channel", "in_process", &["runtime-native"]),
        );

        let mut host = ModuleHost::new(
            ModuleRegistry::new(),
            LoaderConfig {
                search_paths: vec![temp.path().join("src/modules")],
                lock_path: temp.path().join("state/modules.lock"),
                validate: true,
            },
        );

        host.start().expect("host start");

        let runtime = host
            .activation_results()
            .iter()
            .find(|r| r.module_id == "runtime-native")
            .expect("runtime result");
        assert!(runtime.success);
        assert_eq!(runtime.phase, ActivationPhase::Activated);

        let channel = host
            .activation_results()
            .iter()
            .find(|r| r.module_id == "channel-cli")
            .expect("channel result");
        assert!(channel.success);
        assert!(
            channel.diagnostics.starts_with(&[
                "discovered".to_string(),
                "manifest_loaded".to_string(),
                "config_resolved".to_string(),
                "install_state_resolved".to_string(),
                "dependency_validated".to_string(),
                "security_validated".to_string(),
                "runtime_registered".to_string(),
            ]),
            "activation diagnostics should include ordered lifecycle phases"
        );
    }

    #[test]
    fn dependent_module_fails_when_dependency_is_disabled() {
        let temp = tempfile::tempdir().expect("tempdir");
        let runtime_dir = temp.path().join("src/modules/runtimes/runtime-native");
        let channel_dir = temp.path().join("src/modules/channels/channel-cli");

        write_manifest(
            &runtime_dir,
            &canonical_manifest("runtime-native", "runtime", "in_process", &[]),
        );

        write_manifest(
            &channel_dir,
            &canonical_manifest("channel-cli", "channel", "in_process", &["runtime-native"]),
        );

        let lock_path = temp.path().join("state/modules.lock");
        let mut lock = ModulesLock::new();
        lock.add_module(ModuleInstallRecord::bundled_v1(
            "runtime-native",
            "0.1.0",
            false,
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

        host.start().expect("host start");
        let channel = host
            .activation_results()
            .iter()
            .find(|r| r.module_id == "channel-cli")
            .expect("channel result");

        assert!(!channel.success);
        assert_eq!(channel.phase, ActivationPhase::DependencyValidated);
        assert!(channel
            .diagnostics
            .iter()
            .any(|d| d.contains("dependency disabled in modules.lock: runtime-native")));
    }
}
