//! Core installer logic for module lifecycle management.

use crate::core::config::modules_lock::ModulesLock;
use crate::core::contracts::{
    module_manifest::InstallSource, ModuleInstallRecord, ModuleManifest, QuarantineState,
};
use crate::core::installer::artifact_fetch::fetch_artifact;
use crate::core::installer::quarantine::quarantine_artifact;
use crate::core::installer::source_resolver::{resolve_source, ResolvedSource};
use crate::core::installer::verification::{compute_artifact_sha256, verify_module};
use crate::core::lifecycle::load_manifest;
use anyhow::{anyhow, bail, Context, Result};
use semver::{Version, VersionReq};
use std::collections::HashMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// Module installer - handles install/remove/update operations.
pub struct ModuleInstaller {
    /// RedClaw config root for installer state.
    home_root: PathBuf,
    /// modules.lock path
    lock_path: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateResult {
    pub module_id: String,
    pub old_version: String,
    pub new_version: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DoctorStatus {
    Ok,
    Warn,
    Err,
}

impl DoctorStatus {
    fn symbol(self) -> &'static str {
        match self {
            Self::Ok => "✓",
            Self::Warn => "⚠",
            Self::Err => "✗",
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Ok => "OK",
            Self::Warn => "WARN",
            Self::Err => "ERR",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCheck {
    pub status: DoctorStatus,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorCategoryReport {
    pub category: &'static str,
    pub status: DoctorStatus,
    pub checks: Vec<DoctorCheck>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorModuleReport {
    pub module_id: String,
    pub health: DoctorCategoryReport,
    pub dependencies: DoctorCategoryReport,
    pub verification: DoctorCategoryReport,
    pub quarantine: DoctorCategoryReport,
}

impl DoctorModuleReport {
    fn categories(&self) -> [&DoctorCategoryReport; 4] {
        [
            &self.health,
            &self.dependencies,
            &self.verification,
            &self.quarantine,
        ]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctorReport {
    pub modules: Vec<DoctorModuleReport>,
    warning_count: usize,
    error_count: usize,
}

impl DoctorReport {
    fn from_modules(modules: Vec<DoctorModuleReport>) -> Self {
        let mut warning_count = 0;
        let mut error_count = 0;

        for module in &modules {
            for category in module.categories() {
                for check in &category.checks {
                    match check.status {
                        DoctorStatus::Warn => warning_count += 1,
                        DoctorStatus::Err => error_count += 1,
                        DoctorStatus::Ok => {}
                    }
                }
            }
        }

        Self {
            modules,
            warning_count,
            error_count,
        }
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    pub fn error_count(&self) -> usize {
        self.error_count
    }

    pub fn warning_count(&self) -> usize {
        self.warning_count
    }

    pub fn render(&self) -> String {
        let mut lines = vec![
            "Module Health Report".to_string(),
            "====================".to_string(),
            String::new(),
        ];

        for module in &self.modules {
            for category in module.categories() {
                lines.push(format!(
                    "[{}] {} {}",
                    category.category,
                    module.module_id,
                    category.status.label()
                ));
                for check in &category.checks {
                    lines.push(format!("  {} {}", check.status.symbol(), check.detail));
                }
                lines.push(String::new());
            }
        }

        let issues = self.warning_count + self.error_count;
        lines.push(format!(
            "Summary: {} modules checked, {} issue(s) found",
            self.modules.len(),
            issues
        ));

        if self.warning_count > 0 {
            lines.push(format!("  - {} warning(s)", self.warning_count));
        }
        if self.error_count > 0 {
            lines.push(format!("  - {} error(s)", self.error_count));
        }

        lines.join("\n")
    }
}

#[derive(Debug, Clone)]
struct ModuleDoctorState {
    installed_path: PathBuf,
    manifest: Option<ModuleManifest>,
    manifest_error: Option<String>,
}

impl ModuleInstaller {
    /// Create a new installer instance.
    pub fn new(home_root: PathBuf) -> Self {
        let lock_path = home_root.join("modules.lock");
        Self {
            home_root,
            lock_path,
        }
    }

    /// Install a module from source.
    ///
    /// Returns the module ID if successful.
    pub fn install(&self, source: &str, enable: bool) -> Result<String> {
        info!("Installing module from source: {}", source);

        // Step 1: Resolve source
        let resolved = resolve_source(source)?;

        // Step 2: Stage artifact
        let staging_dir = self.home_root.join("modules").join("staging");
        std::fs::create_dir_all(&staging_dir).context("failed to create staging directory")?;

        // Step 3: Fetch artifact
        let artifact_path = fetch_artifact(&resolved, &staging_dir)?;

        // Step 4: Parse and validate manifest
        let manifest_path = artifact_path.join("manifest.json");
        let manifest = match load_manifest(&manifest_path) {
            Ok(m) => m,
            Err(e) => {
                // Quarantine failed artifact
                warn!("Manifest validation failed, quarantining artifact: {}", e);
                let quarantine_dir = self.home_root.join("modules").join("quarantine");
                let quarantine_status = match quarantine_artifact(
                    &artifact_path,
                    &quarantine_dir,
                    &format!("manifest validation failed: {}", e),
                ) {
                    Ok(path) => format!("artifact moved to quarantine: {}", path.display()),
                    Err(quarantine_error) => {
                        warn!(
                            "failed to quarantine artifact after manifest validation failure: {}",
                            quarantine_error
                        );
                        format!("failed to quarantine artifact: {quarantine_error}")
                    }
                };

                return Err(e).context(format!(
                    "failed to load or validate manifest ({quarantine_status})"
                ));
            }
        };

        // Step 5: Verify according to policy
        if let Err(e) = verify_module(&manifest, &artifact_path) {
            // Quarantine failed artifact
            warn!("Verification failed, quarantining artifact: {}", e);
            let quarantine_dir = self.home_root.join("modules").join("quarantine");
            let quarantine_status = match quarantine_artifact(
                &artifact_path,
                &quarantine_dir,
                &format!("verification failed: {}", e),
            ) {
                Ok(path) => format!("artifact moved to quarantine: {}", path.display()),
                Err(quarantine_error) => {
                    warn!(
                        "failed to quarantine artifact after verification failure: {}",
                        quarantine_error
                    );
                    format!("failed to quarantine artifact: {quarantine_error}")
                }
            };

            return Err(e).context(format!("module verification failed ({quarantine_status})"));
        }

        // Step 6: Commit artifact to installed location
        let installed_path = self
            .home_root
            .join("modules")
            .join("installed")
            .join(&manifest.id);

        if installed_path.exists() {
            std::fs::remove_dir_all(&installed_path).context("failed to remove existing module")?;
        }

        std::fs::create_dir_all(installed_path.parent().unwrap())?;
        std::fs::rename(&artifact_path, &installed_path).context("failed to commit artifact")?;

        // Step 7: Write install record atomically (truthful based on source)
        let mut lock = ModulesLock::load(&self.lock_path).unwrap_or_else(|_| ModulesLock::new());

        let (install_source, source_path) = match &resolved {
            ResolvedSource::LocalDir(path) => (InstallSource::LocalDir, Some(path.as_path())),
            ResolvedSource::Archive(path) => (InstallSource::Archive, Some(path.as_path())),
            ResolvedSource::Bundled(_) => (InstallSource::Bundled, None),
        };

        let record =
            ModuleInstallRecord::from_manifest(&manifest, install_source, source_path, enable);
        lock.add_module(record);
        lock.save(&self.lock_path)
            .context("failed to write modules.lock")?;

        info!("Successfully installed module: {}", manifest.id);
        Ok(manifest.id)
    }

    /// Remove a module by ID.
    pub fn remove(&self, module_id: &str) -> Result<()> {
        info!("Removing module: {}", module_id);

        let mut lock = ModulesLock::load(&self.lock_path)?;

        // Check if module exists
        if !lock.modules.iter().any(|m| m.id == module_id) {
            bail!("module {} is not installed", module_id);
        }

        // Remove artifact
        let installed_path = self
            .home_root
            .join("modules")
            .join("installed")
            .join(module_id);

        if installed_path.exists() {
            std::fs::remove_dir_all(&installed_path).context("failed to remove module artifact")?;
        }

        // Remove from lock
        lock.modules.retain(|m| m.id != module_id);
        lock.save(&self.lock_path)?;

        info!("Successfully removed module: {}", module_id);
        Ok(())
    }

    /// List all installed modules.
    pub fn list(&self) -> Result<Vec<ModuleInstallRecord>> {
        let lock = ModulesLock::load(&self.lock_path)?;
        Ok(lock.modules)
    }

    /// Get module info by ID.
    pub fn info(&self, module_id: &str) -> Result<ModuleInstallRecord> {
        let lock = ModulesLock::load(&self.lock_path)?;
        lock.modules
            .into_iter()
            .find(|m| m.id == module_id)
            .ok_or_else(|| anyhow::anyhow!("module {} not found", module_id))
    }

    /// Enable a module by setting `enabled=true` in modules.lock.
    pub fn enable(&self, module_id: &str) -> Result<bool> {
        let mut lock = ModulesLock::load(&self.lock_path)?;

        let (already_enabled, quarantine_state, quarantine_reason, quarantine_since) = {
            let record = lock.get_module(module_id).ok_or_else(|| {
                anyhow::anyhow!("module '{}' not found in modules.lock", module_id)
            })?;
            (
                record.enabled,
                record.quarantine.state,
                record.quarantine.reason.clone(),
                record.quarantine.since.clone(),
            )
        };

        if quarantine_state != QuarantineState::Clear {
            let reason = quarantine_reason.as_deref().unwrap_or("unknown");
            let since = quarantine_since.as_deref().unwrap_or("unknown");
            bail!(
                "cannot enable module '{}': module is quarantined: {} (since {}). Run 'redhorse modules doctor' for details",
                module_id,
                reason,
                since
            );
        }

        if already_enabled {
            return Ok(false);
        }

        let artifact_path = self
            .home_root
            .join("modules")
            .join("installed")
            .join(module_id);
        if !artifact_path.exists() {
            warn!(
                "module '{}' enabled but artifact directory missing: {}",
                module_id,
                artifact_path.display()
            );
        }

        lock.set_enabled(module_id, true)?;
        lock.save(&self.lock_path)
            .context("failed to write modules.lock")?;
        Ok(true)
    }

    /// Disable a module by setting `enabled=false` in modules.lock.
    pub fn disable(&self, module_id: &str) -> Result<bool> {
        let mut lock = ModulesLock::load(&self.lock_path)?;

        let currently_enabled = lock
            .get_module(module_id)
            .map(|record| record.enabled)
            .ok_or_else(|| anyhow::anyhow!("module '{}' not found in modules.lock", module_id))?;

        if !currently_enabled {
            return Ok(false);
        }

        let enabled_dependents = self.find_enabled_dependents(module_id, &lock);
        for dependent in &enabled_dependents {
            eprintln!(
                "Warning: module '{}' is required by enabled module '{}'",
                module_id, dependent
            );
        }
        if !enabled_dependents.is_empty() {
            eprintln!("Disabling may cause dependency issues");
        }

        lock.set_enabled(module_id, false)?;
        lock.save(&self.lock_path)
            .context("failed to write modules.lock")?;
        Ok(true)
    }

    /// Update a single module or all installed modules.
    pub fn update(&self, module_id: Option<&str>, all: bool) -> Result<Vec<UpdateResult>> {
        if all && module_id.is_some() {
            bail!("modules update does not accept <module-id> together with --all");
        }

        if !all && module_id.is_none() {
            bail!("modules update requires <module-id> or --all");
        }

        let targets = if all {
            let lock = ModulesLock::load(&self.lock_path)?;
            lock.modules.into_iter().map(|record| record.id).collect()
        } else {
            vec![module_id.expect("validated module_id").to_string()]
        };

        let mut results = Vec::new();
        let mut failures = Vec::new();

        for target in targets {
            match self.update_single(&target) {
                Ok(result) => results.push(result),
                Err(error) => {
                    if !all {
                        return Err(error);
                    }
                    failures.push((target, format!("{error:#}")));
                }
            }
        }

        if !failures.is_empty() {
            let mut message = format!("{} module(s) failed to update", failures.len());
            for (failed_module, failure) in failures {
                let summary = failure.lines().next().unwrap_or(failure.as_str());
                let _ = write!(message, "\n- {failed_module}: {summary}");
            }
            bail!(message);
        }

        Ok(results)
    }

    fn update_single(&self, module_id: &str) -> Result<UpdateResult> {
        let mut lock = ModulesLock::load(&self.lock_path)?;
        let module_index = lock
            .modules
            .iter()
            .position(|record| record.id == module_id)
            .ok_or_else(|| anyhow!("module '{}' not found in modules.lock", module_id))?;

        let current = lock.modules[module_index].clone();
        let previous_version = current.version.clone();

        let source_hint = current
            .source
            .path
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .or_else(|| {
                current
                    .source
                    .uri
                    .as_deref()
                    .filter(|value| !value.trim().is_empty())
            })
            .ok_or_else(|| anyhow!("module '{}' has no source.path or source.uri", module_id))?;

        let resolved = resolve_source(source_hint).with_context(|| {
            format!("failed to resolve update source for module '{}'", module_id)
        })?;

        let staging_root = self.home_root.join("modules").join("staging");
        std::fs::create_dir_all(&staging_root).context("failed to create staging directory")?;

        let module_staging_dir = staging_root.join(format!("{}-update", module_id));
        if module_staging_dir.exists() {
            std::fs::remove_dir_all(&module_staging_dir)
                .context("failed to clean module staging directory")?;
        }
        std::fs::create_dir_all(&module_staging_dir)
            .context("failed to create module staging directory")?;

        let staged_artifact_path =
            fetch_artifact(&resolved, &module_staging_dir).with_context(|| {
                format!("failed to fetch update artifact for module '{}'", module_id)
            })?;

        let staged_manifest_path = staged_artifact_path.join("manifest.json");
        let manifest = match load_manifest(&staged_manifest_path) {
            Ok(manifest) => manifest,
            Err(error) => {
                let quarantine_status = self.quarantine_update_candidate(
                    module_id,
                    &staged_artifact_path,
                    &format!("manifest validation failed: {error}"),
                );
                bail!(
                    "update failed for '{}': {}\nOld version ({}) remains active ({})",
                    module_id,
                    error,
                    previous_version,
                    quarantine_status
                );
            }
        };

        if manifest.id != module_id {
            let quarantine_status = self.quarantine_update_candidate(
                module_id,
                &staged_artifact_path,
                &format!(
                    "manifest id mismatch: expected '{}', found '{}'",
                    module_id, manifest.id
                ),
            );
            bail!(
                "update failed for '{}': manifest id mismatch (found '{}')\nOld version ({}) remains active ({})",
                module_id,
                manifest.id,
                previous_version,
                quarantine_status
            );
        }

        if let Err(error) = verify_module(&manifest, &staged_artifact_path) {
            let quarantine_status = self.quarantine_update_candidate(
                module_id,
                &staged_artifact_path,
                &format!("verification failed: {error}"),
            );
            bail!(
                "update failed for '{}': {}\nOld version ({}) remains active ({})",
                module_id,
                error,
                previous_version,
                quarantine_status
            );
        }

        let checksum = compute_artifact_sha256(&staged_artifact_path)
            .map(|digest| format!("sha256:{digest}"))
            .context("failed to compute staged artifact checksum")?;

        let installed_root = self.home_root.join("modules").join("installed");
        std::fs::create_dir_all(&installed_root).context("failed to create installed directory")?;

        let installed_path = installed_root.join(module_id);
        let rollback_path = staging_root.join(format!("{}-rollback", module_id));
        if rollback_path.exists() {
            std::fs::remove_dir_all(&rollback_path)
                .context("failed to clean previous rollback snapshot")?;
        }

        let had_previous_artifact = installed_path.exists();
        if had_previous_artifact {
            std::fs::rename(&installed_path, &rollback_path)
                .context("failed to stage previous artifact for rollback")?;
        }

        if let Err(commit_error) = std::fs::rename(&staged_artifact_path, &installed_path) {
            if had_previous_artifact && rollback_path.exists() {
                let _ = std::fs::rename(&rollback_path, &installed_path);
            }
            return Err(commit_error).context(format!(
                "failed to commit update for module '{}'",
                module_id
            ));
        }

        let source_path = match &resolved {
            ResolvedSource::LocalDir(path) | ResolvedSource::Archive(path) => Some(path.as_path()),
            ResolvedSource::Bundled(_) => None,
        };

        let mut updated_record = ModuleInstallRecord::from_manifest(
            &manifest,
            current.install.source.clone(),
            source_path,
            current.enabled,
        );
        updated_record.source = current.source.clone();
        updated_record.previous_version = Some(current.version.clone());
        updated_record.previous_checksum = current.checksum.clone();
        updated_record.checksum = Some(checksum);
        updated_record.signature = current.signature.clone();

        lock.modules[module_index] = updated_record;

        if let Err(lock_error) = lock.save(&self.lock_path) {
            if installed_path.exists() {
                let quarantine_dir = self.home_root.join("modules").join("quarantine");
                let reason = format!(
                    "update committed artifact for '{}' but lockfile write failed: {}",
                    module_id, lock_error
                );
                if let Err(quarantine_error) =
                    quarantine_artifact(&installed_path, &quarantine_dir, &reason)
                {
                    warn!(
                        "failed to quarantine committed artifact for '{}': {}",
                        module_id, quarantine_error
                    );
                    let _ = std::fs::remove_dir_all(&installed_path);
                }
            }

            if had_previous_artifact && rollback_path.exists() {
                std::fs::rename(&rollback_path, &installed_path)
                    .context("failed to restore previous artifact after lockfile error")?;
            }

            return Err(lock_error).context(format!(
                "failed to write modules.lock for module '{}'",
                module_id
            ));
        }

        if had_previous_artifact && rollback_path.exists() {
            std::fs::remove_dir_all(&rollback_path)
                .context("failed to clean rollback snapshot after successful update")?;
        }

        Ok(UpdateResult {
            module_id: module_id.to_string(),
            old_version: previous_version,
            new_version: manifest.version,
        })
    }

    /// Run health diagnostics over installed modules.
    pub fn doctor(&self) -> Result<DoctorReport> {
        let lock = ModulesLock::load(&self.lock_path).context("cannot read modules.lock")?;

        let mut states = HashMap::new();
        for record in &lock.modules {
            states.insert(record.id.clone(), self.build_module_state(record));
        }

        let mut module_reports = Vec::new();
        for record in &lock.modules {
            let state = states
                .get(&record.id)
                .ok_or_else(|| anyhow!("missing doctor state for module '{}'", record.id))?;

            let health = self.build_health_report(record, state);
            let dependencies = self.build_dependencies_report(record, &lock.modules, &states);
            let verification = self.build_verification_report(record, state);
            let quarantine = self.build_quarantine_report(record);

            module_reports.push(DoctorModuleReport {
                module_id: record.id.clone(),
                health,
                dependencies,
                verification,
                quarantine,
            });
        }

        Ok(DoctorReport::from_modules(module_reports))
    }

    fn build_module_state(&self, record: &ModuleInstallRecord) -> ModuleDoctorState {
        let installed_path = self
            .home_root
            .join("modules")
            .join("installed")
            .join(&record.id);
        let manifest_path = installed_path.join("manifest.json");

        let (manifest, manifest_error) = if manifest_path.exists() {
            match load_manifest(&manifest_path) {
                Ok(manifest) => (Some(manifest), None),
                Err(error) => (None, Some(error.to_string())),
            }
        } else {
            (None, None)
        };

        ModuleDoctorState {
            installed_path,
            manifest,
            manifest_error,
        }
    }

    fn build_health_report(
        &self,
        record: &ModuleInstallRecord,
        state: &ModuleDoctorState,
    ) -> DoctorCategoryReport {
        let mut checks = vec![DoctorCheck {
            status: DoctorStatus::Ok,
            detail: "modules.lock readable".to_string(),
        }];

        if state.installed_path.exists() {
            checks.push(DoctorCheck {
                status: DoctorStatus::Ok,
                detail: format!(
                    "artifact directory exists: {}",
                    state.installed_path.display()
                ),
            });
        } else {
            checks.push(DoctorCheck {
                status: DoctorStatus::Err,
                detail: format!(
                    "artifact directory missing: {}",
                    state.installed_path.display()
                ),
            });
        }

        let manifest_path = state.installed_path.join("manifest.json");
        if !manifest_path.exists() {
            checks.push(DoctorCheck {
                status: DoctorStatus::Err,
                detail: "manifest.json missing".to_string(),
            });
        } else if let Some(error) = &state.manifest_error {
            checks.push(DoctorCheck {
                status: DoctorStatus::Err,
                detail: format!("manifest.json unreadable: {error}"),
            });
        } else {
            checks.push(DoctorCheck {
                status: DoctorStatus::Ok,
                detail: "manifest.json present and valid".to_string(),
            });
        }

        if let Some(manifest) = &state.manifest {
            if manifest.id == record.id {
                checks.push(DoctorCheck {
                    status: DoctorStatus::Ok,
                    detail: "manifest id matches modules.lock record".to_string(),
                });
            } else {
                checks.push(DoctorCheck {
                    status: DoctorStatus::Err,
                    detail: format!(
                        "manifest id mismatch: expected '{}', found '{}'",
                        record.id, manifest.id
                    ),
                });
            }

            if manifest.artifact.kind == crate::core::contracts::ArtifactKind::Bundled {
                checks.push(DoctorCheck {
                    status: DoctorStatus::Ok,
                    detail: format!(
                        "artifact entry is logical for bundled modules: {}",
                        manifest.artifact.entry
                    ),
                });
            } else {
                let entry_path = state.installed_path.join(&manifest.artifact.entry);
                if entry_path.exists() {
                    checks.push(DoctorCheck {
                        status: DoctorStatus::Ok,
                        detail: format!("artifact entry exists: {}", manifest.artifact.entry),
                    });
                } else {
                    checks.push(DoctorCheck {
                        status: DoctorStatus::Err,
                        detail: format!("artifact entry missing: {}", entry_path.display()),
                    });
                }
            }
        }

        make_category("HEALTH", checks)
    }

    fn build_dependencies_report(
        &self,
        record: &ModuleInstallRecord,
        lock_modules: &[ModuleInstallRecord],
        states: &HashMap<String, ModuleDoctorState>,
    ) -> DoctorCategoryReport {
        let mut checks = Vec::new();

        match states
            .get(&record.id)
            .and_then(|state| state.manifest.as_ref())
        {
            Some(manifest) => {
                if manifest.dependencies.is_empty() {
                    checks.push(DoctorCheck {
                        status: DoctorStatus::Ok,
                        detail: "no declared dependencies".to_string(),
                    });
                }

                for dependency in &manifest.dependencies {
                    let dependency_record = lock_modules
                        .iter()
                        .find(|candidate| candidate.id == dependency.module_id);

                    match dependency_record {
                        Some(candidate) => {
                            checks.push(DoctorCheck {
                                status: DoctorStatus::Ok,
                                detail: format!(
                                    "dependency installed: {} ({})",
                                    dependency.module_id, candidate.version
                                ),
                            });

                            if let Some(version_req) = &dependency.version {
                                match (
                                    VersionReq::parse(version_req),
                                    Version::parse(&candidate.version),
                                ) {
                                    (Ok(requirement), Ok(version)) => {
                                        if requirement.matches(&version) {
                                            checks.push(DoctorCheck {
                                                status: DoctorStatus::Ok,
                                                detail: format!(
                                                    "dependency version satisfied: {} {}",
                                                    dependency.module_id, version_req
                                                ),
                                            });
                                        } else {
                                            checks.push(DoctorCheck {
                                                status: DoctorStatus::Err,
                                                detail: format!(
                                                    "dependency version mismatch: {} requires {}, installed {}",
                                                    dependency.module_id,
                                                    version_req,
                                                    candidate.version
                                                ),
                                            });
                                        }
                                    }
                                    (Err(error), _) => {
                                        checks.push(DoctorCheck {
                                            status: DoctorStatus::Err,
                                            detail: format!(
                                                "dependency version requirement invalid for {}: {}",
                                                dependency.module_id, error
                                            ),
                                        });
                                    }
                                    (_, Err(error)) => {
                                        checks.push(DoctorCheck {
                                            status: DoctorStatus::Err,
                                            detail: format!(
                                                "installed dependency version invalid for {}: {}",
                                                dependency.module_id, error
                                            ),
                                        });
                                    }
                                }
                            }

                            if record.enabled && !dependency.optional && !candidate.enabled {
                                checks.push(DoctorCheck {
                                    status: DoctorStatus::Warn,
                                    detail: format!(
                                        "enabled module depends on disabled module '{}'",
                                        dependency.module_id
                                    ),
                                });
                            }
                        }
                        None => {
                            let status = if dependency.optional {
                                DoctorStatus::Warn
                            } else {
                                DoctorStatus::Err
                            };
                            checks.push(DoctorCheck {
                                status,
                                detail: format!("dependency missing: {}", dependency.module_id),
                            });
                        }
                    }
                }
            }
            None => {
                checks.push(DoctorCheck {
                    status: DoctorStatus::Warn,
                    detail: "dependency checks limited: manifest unavailable".to_string(),
                });
            }
        }

        if !record.enabled {
            for module in lock_modules {
                if !module.enabled || module.id == record.id {
                    continue;
                }

                let Some(other_manifest) = states
                    .get(&module.id)
                    .and_then(|state| state.manifest.as_ref())
                else {
                    continue;
                };

                let depends_on_record = other_manifest
                    .dependencies
                    .iter()
                    .any(|dependency| !dependency.optional && dependency.module_id == record.id);
                if depends_on_record {
                    checks.push(DoctorCheck {
                        status: DoctorStatus::Warn,
                        detail: format!("disabled, but required by enabled module '{}'", module.id),
                    });
                }
            }
        }

        make_category("DEPENDENCIES", checks)
    }

    fn build_verification_report(
        &self,
        record: &ModuleInstallRecord,
        state: &ModuleDoctorState,
    ) -> DoctorCategoryReport {
        let mut checks = vec![DoctorCheck {
            status: DoctorStatus::Ok,
            detail: format!("trust tier: {:?}", record.trust.tier),
        }];

        match record.trust.verification_state.as_str() {
            "verified" => checks.push(DoctorCheck {
                status: DoctorStatus::Ok,
                detail: "verification state: verified".to_string(),
            }),
            "unverified" => checks.push(DoctorCheck {
                status: DoctorStatus::Warn,
                detail: "verification state: unverified".to_string(),
            }),
            other => checks.push(DoctorCheck {
                status: DoctorStatus::Err,
                detail: format!("verification state: {other}"),
            }),
        }

        match &record.checksum {
            Some(expected_checksum) => {
                if state.installed_path.exists() {
                    match compute_artifact_sha256(&state.installed_path) {
                        Ok(actual_checksum) => {
                            if checksum_matches(expected_checksum, &actual_checksum) {
                                checks.push(DoctorCheck {
                                    status: DoctorStatus::Ok,
                                    detail: "checksum match: verified".to_string(),
                                });
                            } else {
                                checks.push(DoctorCheck {
                                    status: DoctorStatus::Err,
                                    detail: format!(
                                        "checksum mismatch: expected {}, got sha256:{}",
                                        expected_checksum, actual_checksum
                                    ),
                                });
                            }
                        }
                        Err(error) => checks.push(DoctorCheck {
                            status: DoctorStatus::Err,
                            detail: format!("checksum verification failed: {error}"),
                        }),
                    }
                } else {
                    checks.push(DoctorCheck {
                        status: DoctorStatus::Err,
                        detail: "checksum unavailable: artifact directory missing".to_string(),
                    });
                }
            }
            None => checks.push(DoctorCheck {
                status: DoctorStatus::Warn,
                detail: "checksum missing in modules.lock".to_string(),
            }),
        }

        make_category("VERIFICATION", checks)
    }

    fn build_quarantine_report(&self, record: &ModuleInstallRecord) -> DoctorCategoryReport {
        let mut checks = Vec::new();

        match record.quarantine.state {
            QuarantineState::Clear => {
                checks.push(DoctorCheck {
                    status: DoctorStatus::Ok,
                    detail: "quarantine state: clear".to_string(),
                });
            }
            QuarantineState::Quarantined => {
                checks.push(DoctorCheck {
                    status: DoctorStatus::Err,
                    detail: "quarantine state: quarantined".to_string(),
                });

                checks.push(DoctorCheck {
                    status: DoctorStatus::Err,
                    detail: format!(
                        "reason: {}",
                        record.quarantine.reason.as_deref().unwrap_or("unknown")
                    ),
                });
                checks.push(DoctorCheck {
                    status: DoctorStatus::Err,
                    detail: format!(
                        "since: {}",
                        record.quarantine.since.as_deref().unwrap_or("unknown")
                    ),
                });

                if self.has_matching_quarantine_artifact(record) {
                    checks.push(DoctorCheck {
                        status: DoctorStatus::Ok,
                        detail: "matching quarantine artifact found".to_string(),
                    });
                } else {
                    checks.push(DoctorCheck {
                        status: DoctorStatus::Err,
                        detail: "matching quarantine artifact not found".to_string(),
                    });
                }
            }
        }

        make_category("QUARANTINE", checks)
    }

    fn has_matching_quarantine_artifact(&self, record: &ModuleInstallRecord) -> bool {
        let quarantine_dir = self.home_root.join("modules").join("quarantine");
        if !quarantine_dir.exists() {
            return false;
        }

        let reason_hint = record.quarantine.reason.as_deref().unwrap_or_default();

        let Ok(entries) = std::fs::read_dir(&quarantine_dir) else {
            return false;
        };

        for entry in entries.flatten() {
            let entry_path = entry.path();
            if !entry_path.is_dir() {
                continue;
            }

            let name = entry.file_name().to_string_lossy().to_string();
            if name.contains(&record.id) {
                return true;
            }

            let reason_path = entry_path.join("QUARANTINE_REASON.txt");
            if let Ok(contents) = std::fs::read_to_string(reason_path) {
                if contents.contains(&record.id)
                    || (!reason_hint.is_empty() && contents.contains(reason_hint))
                {
                    return true;
                }
            }
        }

        false
    }

    fn quarantine_update_candidate(
        &self,
        module_id: &str,
        candidate_path: &Path,
        reason: &str,
    ) -> String {
        if !candidate_path.exists() {
            return "candidate artifact unavailable for quarantine".to_string();
        }

        let quarantine_dir = self.home_root.join("modules").join("quarantine");
        let contextual_reason = format!("update candidate for '{}': {}", module_id, reason);

        match quarantine_artifact(candidate_path, &quarantine_dir, &contextual_reason) {
            Ok(path) => format!("candidate quarantined at {}", path.display()),
            Err(error) => {
                warn!(
                    "failed to quarantine update candidate for '{}': {}",
                    module_id, error
                );
                format!("failed to quarantine candidate: {error}")
            }
        }
    }

    fn find_enabled_dependents(&self, module_id: &str, lock: &ModulesLock) -> Vec<String> {
        let mut dependents = Vec::new();
        let installed_root = self.home_root.join("modules").join("installed");

        for module in &lock.modules {
            if module.id == module_id || !module.enabled {
                continue;
            }

            let manifest_path = installed_root.join(&module.id).join("manifest.json");
            if !manifest_path.exists() {
                continue;
            }

            match load_manifest(&manifest_path) {
                Ok(manifest) => {
                    let depends_on_target = manifest
                        .dependencies
                        .iter()
                        .any(|dep| !dep.optional && dep.module_id == module_id);
                    if depends_on_target {
                        dependents.push(module.id.clone());
                    }
                }
                Err(error) => {
                    warn!(
                        "failed to inspect dependencies for module '{}': {}",
                        module.id, error
                    );
                }
            }
        }

        dependents
    }
}

fn make_category(category: &'static str, mut checks: Vec<DoctorCheck>) -> DoctorCategoryReport {
    if checks.is_empty() {
        checks.push(DoctorCheck {
            status: DoctorStatus::Ok,
            detail: "no issues detected".to_string(),
        });
    }

    DoctorCategoryReport {
        category,
        status: category_status(&checks),
        checks,
    }
}

fn category_status(checks: &[DoctorCheck]) -> DoctorStatus {
    checks
        .iter()
        .map(|check| check.status)
        .max()
        .unwrap_or(DoctorStatus::Ok)
}

fn checksum_matches(expected: &str, actual_sha256: &str) -> bool {
    let normalized_expected = expected
        .trim()
        .strip_prefix("sha256:")
        .unwrap_or(expected.trim());
    normalized_expected.eq_ignore_ascii_case(actual_sha256)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::contracts::InstallSource;

    fn create_valid_module(module_dir: &std::path::Path, id: &str, version: &str) {
        std::fs::create_dir_all(module_dir).expect("create module directory");

        let manifest = format!(
            r#"{{
    "id": "{id}",
    "name": "{id}",
    "version": "{version}",
    "kind": "runtime",
    "engine": {{ "redhorse": ">=0.1.0 <0.2.0" }},
    "artifact": {{ "kind": "bundled", "entry": "modules/runtimes/{id}" }},
    "execution": {{ "mode": "in_process" }},
    "trust": {{ "required": "official" }},
    "capabilities": {{ "requested": [], "parameterized": [] }},
    "dependencies": [],
    "config": {{ "schema": {{"type": "object"}}, "defaultFragment": {{}} }},
    "activation": {{ "events": ["startup"], "safeModeEligible": true }},
    "install": {{ "source": "bundled" }}
}}"#
        );

        std::fs::write(module_dir.join("manifest.json"), manifest).expect("write manifest");
    }

    #[test]
    fn install_records_original_local_source_path() {
        let home = tempfile::tempdir().expect("temp home");
        let source_root = tempfile::tempdir().expect("temp source");
        let module_dir = source_root.path().join("local-source-module");
        create_valid_module(&module_dir, "local-source-module", "0.1.0");

        let installer = ModuleInstaller::new(home.path().to_path_buf());
        installer
            .install(module_dir.to_str().expect("utf8 module path"), false)
            .expect("install should succeed");

        let lock_path = home.path().join("modules.lock");
        let lock = ModulesLock::load(&lock_path).expect("load modules.lock");
        let record = lock
            .modules
            .iter()
            .find(|record| record.id == "local-source-module")
            .expect("installed record present");

        assert_eq!(record.install.source, InstallSource::LocalDir);
        let expected_path = module_dir.to_string_lossy().to_string();
        assert_eq!(record.source.path.as_deref(), Some(expected_path.as_str()));
    }

    #[test]
    fn install_reports_quarantine_path_when_manifest_validation_fails() {
        let home = tempfile::tempdir().expect("temp home");
        let source_root = tempfile::tempdir().expect("temp source");
        let invalid_module = source_root.path().join("broken-module");
        std::fs::create_dir_all(&invalid_module).expect("create invalid module dir");
        std::fs::write(
            invalid_module.join("manifest.json"),
            r#"{"id":"broken-module","version":"0.1.0"}"#,
        )
        .expect("write invalid manifest");

        let installer = ModuleInstaller::new(home.path().to_path_buf());
        let error = installer
            .install(invalid_module.to_str().expect("utf8 module path"), false)
            .expect_err("invalid manifest should fail");

        let error_message = format!("{error:#}");
        assert!(
            error_message.contains("artifact moved to quarantine:"),
            "expected quarantine path in error, got: {error_message}"
        );

        let quarantine_dir = home.path().join("modules").join("quarantine");
        let quarantined_entries = std::fs::read_dir(&quarantine_dir)
            .expect("read quarantine dir")
            .collect::<std::io::Result<Vec<_>>>()
            .expect("collect quarantine entries");
        assert_eq!(
            quarantined_entries.len(),
            1,
            "expected one quarantine entry"
        );
        assert!(
            quarantined_entries[0]
                .path()
                .join("QUARANTINE_REASON.txt")
                .exists(),
            "expected quarantine reason file"
        );
    }
}
