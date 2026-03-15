//! Integration tests for module installer.
//!
//! Tests the full install -> activate -> uninstall cycle.

use redclaw::core::config::modules_lock::ModulesLock;
use redclaw::core::contracts::QuarantineState;
use redclaw::core::installer::verification::compute_artifact_sha256;
use redclaw::core::installer::ModuleInstaller;
use redclaw::core::lifecycle::load_manifest;
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Create a test module directory with manifest.
fn create_test_module(dir: &Path, id: &str, version: &str) -> PathBuf {
    create_test_module_with_dependencies(dir, id, version, &[])
}

fn create_test_module_with_dependencies(
    dir: &Path,
    id: &str,
    version: &str,
    dependencies: &[&str],
) -> PathBuf {
    let module_dir = dir.join(id);
    fs::create_dir_all(&module_dir).expect("failed to create module dir");

    let dependencies = dependencies
        .iter()
        .map(|dep| format!(r#"{{"id":"{dep}"}}"#))
        .collect::<Vec<_>>()
        .join(", ");

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
            "dependencies": [{dependencies}],
            "config": {{ "schema": {{"type": "object"}}, "defaultFragment": {{}} }},
            "activation": {{ "events": ["startup"], "safeModeEligible": true }},
            "install": {{ "source": "bundled" }}
        }}"#
    );

    fs::write(module_dir.join("manifest.json"), manifest).expect("failed to write manifest");
    module_dir
}

/// Create a test Redhorse home directory structure.
fn create_test_home() -> (TempDir, PathBuf) {
    let temp = TempDir::new().expect("failed to create temp dir");
    let home = temp.path().to_path_buf();

    // Create directory structure
    fs::create_dir_all(home.join("modules").join("staging")).unwrap();
    fs::create_dir_all(home.join("modules").join("installed")).unwrap();
    fs::create_dir_all(home.join("modules").join("quarantine")).unwrap();
    fs::create_dir_all(home.join("state")).unwrap();

    (temp, home)
}

#[test]
fn test_installer_install_local_dir() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create a test module
    let test_module_dir = PathBuf::from("/tmp/test-modules");
    let module_path = create_test_module(&test_module_dir, "test-module", "0.1.0");

    // Install the module
    let result = installer.install(module_path.to_str().unwrap(), false);

    assert!(result.is_ok(), "install should succeed: {:?}", result.err());
    let module_id = result.unwrap();
    assert_eq!(module_id, "test-module");

    // Verify module is in installed directory
    let installed_path = home.join("modules").join("installed").join(&module_id);
    assert!(installed_path.exists(), "installed module should exist");

    // Verify module is in modules.lock
    let lock_path = home.join("modules.lock");
    assert!(lock_path.exists(), "modules.lock should exist");

    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 1);
    assert_eq!(lock.modules[0].id, "test-module");
    assert_eq!(lock.modules[0].version, "0.1.0");
    assert!(
        !lock.modules[0].enabled,
        "module should be disabled by default"
    );

    // Cleanup
    fs::remove_dir_all(&test_module_dir).ok();
}

#[test]
fn test_installer_install_twice_updates_record() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create test module v1
    let test_module_dir = PathBuf::from("/tmp/test-modules-v1");
    let module_path_v1 = create_test_module(&test_module_dir, "test-module", "0.1.0");

    // Install v1
    installer
        .install(module_path_v1.to_str().unwrap(), false)
        .unwrap();

    // Create test module v2
    let test_module_dir_v2 = PathBuf::from("/tmp/test-modules-v2");
    let module_path_v2 = create_test_module(&test_module_dir_v2, "test-module", "0.2.0");

    // Install v2 (should update)
    installer
        .install(module_path_v2.to_str().unwrap(), false)
        .unwrap();

    // Verify version updated
    let lock_path = home.join("modules.lock");
    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 1);
    assert_eq!(lock.modules[0].version, "0.2.0");

    // Cleanup
    fs::remove_dir_all(&test_module_dir).ok();
    fs::remove_dir_all(&test_module_dir_v2).ok();
}

#[test]
fn test_installer_remove() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create and install test module
    let test_module_dir = PathBuf::from("/tmp/test-modules-remove");
    let module_path = create_test_module(&test_module_dir, "test-module", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), false)
        .unwrap();

    // Verify installed
    let installed_path = home.join("modules").join("installed").join("test-module");
    assert!(installed_path.exists(), "module should be installed");

    // Remove the module
    let result = installer.remove("test-module");
    assert!(result.is_ok(), "remove should succeed: {:?}", result.err());

    // Verify removed from filesystem
    assert!(
        !installed_path.exists(),
        "installed module should be removed"
    );

    // Verify removed from lock
    let lock_path = home.join("modules.lock");
    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 0);

    // Cleanup
    fs::remove_dir_all(&test_module_dir).ok();
}

#[test]
fn test_installer_list() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create and install multiple modules
    let test_module_dir = PathBuf::from("/tmp/test-modules-list");

    let module1 = create_test_module(&test_module_dir, "module-1", "0.1.0");
    let module2 = create_test_module(&test_module_dir, "module-2", "0.2.0");
    let module3 = create_test_module(&test_module_dir, "module-3", "0.3.0");

    installer.install(module1.to_str().unwrap(), false).unwrap();
    installer.install(module2.to_str().unwrap(), true).unwrap();
    installer.install(module3.to_str().unwrap(), false).unwrap();

    // List modules
    let modules = installer.list().expect("list should succeed");
    assert_eq!(modules.len(), 3);

    let ids: Vec<&String> = modules.iter().map(|m| &m.id).collect();
    assert!(ids.contains(&&"module-1".to_string()));
    assert!(ids.contains(&&"module-2".to_string()));
    assert!(ids.contains(&&"module-3".to_string()));

    // Cleanup
    fs::remove_dir_all(&test_module_dir).ok();
}

#[test]
fn test_installer_info() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create and install test module
    let test_module_dir = PathBuf::from("/tmp/test-modules-info");
    let module_path = create_test_module(&test_module_dir, "test-module", "1.2.3");
    installer
        .install(module_path.to_str().unwrap(), true)
        .unwrap();

    // Get module info
    let info = installer.info("test-module").expect("info should succeed");
    assert_eq!(info.id, "test-module");
    assert_eq!(info.version, "1.2.3");
    assert!(info.enabled);

    // Test non-existent module
    let result = installer.info("non-existent");
    assert!(result.is_err(), "info should fail for non-existent module");

    // Cleanup
    fs::remove_dir_all(&test_module_dir).ok();
}

#[test]
fn test_installer_enable_sets_enabled_flag() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "toggle-module", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), false)
        .expect("install should succeed");

    let changed = installer
        .enable("toggle-module")
        .expect("enable should succeed");
    assert!(changed, "enable should mutate disabled module");

    let lock = ModulesLock::load(&home.join("modules.lock")).expect("load modules.lock");
    let record = lock
        .modules
        .iter()
        .find(|record| record.id == "toggle-module")
        .expect("record should exist");
    assert!(record.enabled, "module should be enabled after enable()");
}

#[test]
fn test_installer_enable_is_idempotent() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "idempotent-enable", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), false)
        .expect("install should succeed");

    let first = installer
        .enable("idempotent-enable")
        .expect("first enable should succeed");
    let second = installer
        .enable("idempotent-enable")
        .expect("second enable should succeed");

    assert!(first, "first enable should change state");
    assert!(!second, "second enable should be idempotent");

    let lock = ModulesLock::load(&home.join("modules.lock")).expect("load modules.lock");
    let record = lock
        .modules
        .iter()
        .find(|record| record.id == "idempotent-enable")
        .expect("record should exist");
    assert!(record.enabled, "module should remain enabled");
}

#[test]
fn test_installer_enable_fails_for_missing_module() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home);

    let error = installer
        .enable("missing-module")
        .expect_err("enable should fail for missing module");
    assert!(
        format!("{error:#}").contains("not found"),
        "error should mention missing module"
    );
}

#[test]
fn test_installer_enable_fails_for_quarantined_module() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "quarantined-module", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), false)
        .expect("install should succeed");

    let lock_path = home.join("modules.lock");
    let mut lock = ModulesLock::load(&lock_path).expect("load modules.lock");
    let record = lock
        .modules
        .iter_mut()
        .find(|record| record.id == "quarantined-module")
        .expect("record should exist");
    record.quarantine.state = QuarantineState::Quarantined;
    record.quarantine.reason = Some("verification_failed".to_string());
    record.quarantine.since = Some("2026-03-15T10:30:00Z".to_string());
    lock.save(&lock_path).expect("save modules.lock");

    let error = installer
        .enable("quarantined-module")
        .expect_err("enable should fail for quarantined module");
    assert!(
        format!("{error:#}").contains("quarantined"),
        "error should mention quarantine state"
    );
}

#[test]
fn test_installer_disable_sets_enabled_flag() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "disable-module", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), true)
        .expect("install should succeed");

    let changed = installer
        .disable("disable-module")
        .expect("disable should succeed");
    assert!(changed, "disable should mutate enabled module");

    let lock = ModulesLock::load(&home.join("modules.lock")).expect("load modules.lock");
    let record = lock
        .modules
        .iter()
        .find(|record| record.id == "disable-module")
        .expect("record should exist");
    assert!(!record.enabled, "module should be disabled after disable()");
}

#[test]
fn test_installer_disable_is_idempotent() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "idempotent-disable", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), false)
        .expect("install should succeed");

    let first = installer
        .disable("idempotent-disable")
        .expect("first disable should succeed");
    let second = installer
        .disable("idempotent-disable")
        .expect("second disable should succeed");

    assert!(!first, "already disabled module should be idempotent");
    assert!(!second, "second disable should remain idempotent");
}

#[test]
fn test_installer_disable_fails_for_missing_module() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home);

    let error = installer
        .disable("missing-module")
        .expect_err("disable should fail for missing module");
    assert!(
        format!("{error:#}").contains("not found"),
        "error should mention missing module"
    );
}

#[test]
fn test_installer_disable_warns_only_when_enabled_dependents_exist() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let core_path =
        create_test_module_with_dependencies(source_root.path(), "runtime-core", "0.1.0", &[]);
    let dependent_path = create_test_module_with_dependencies(
        source_root.path(),
        "tool-dependent",
        "0.1.0",
        &["runtime-core"],
    );

    installer
        .install(core_path.to_str().unwrap(), true)
        .expect("runtime-core install should succeed");
    installer
        .install(dependent_path.to_str().unwrap(), true)
        .expect("tool-dependent install should succeed");

    let changed = installer
        .disable("runtime-core")
        .expect("disable should not fail when dependents are enabled");
    assert!(changed, "disable should still mutate target module state");

    let lock = ModulesLock::load(&home.join("modules.lock")).expect("load modules.lock");
    let runtime_core = lock
        .modules
        .iter()
        .find(|record| record.id == "runtime-core")
        .expect("runtime-core record should exist");
    let tool_dependent = lock
        .modules
        .iter()
        .find(|record| record.id == "tool-dependent")
        .expect("tool-dependent record should exist");

    assert!(!runtime_core.enabled, "runtime-core should be disabled");
    assert!(
        tool_dependent.enabled,
        "dependent module should remain enabled after warning-only disable"
    );
}

#[test]
fn test_installer_update_single_preserves_rollback_metadata() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "update-module", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), true)
        .expect("install should succeed");

    let lock_path = home.join("modules.lock");
    let mut lock = ModulesLock::load(&lock_path).expect("load modules.lock");
    let record = lock
        .modules
        .iter_mut()
        .find(|record| record.id == "update-module")
        .expect("record should exist");
    record.checksum = Some("sha256:old-checksum".to_string());
    lock.save(&lock_path).expect("save modules.lock");

    create_test_module(source_root.path(), "update-module", "0.2.0");

    let results = installer
        .update(Some("update-module"), false)
        .expect("update should succeed");

    assert_eq!(results.len(), 1, "expected exactly one update result");
    assert_eq!(results[0].module_id, "update-module");
    assert_eq!(results[0].old_version, "0.1.0");
    assert_eq!(results[0].new_version, "0.2.0");

    let lock = ModulesLock::load(&lock_path).expect("load modules.lock");
    let updated = lock
        .modules
        .iter()
        .find(|record| record.id == "update-module")
        .expect("updated record should exist");

    assert_eq!(updated.version, "0.2.0");
    assert_eq!(updated.previous_version.as_deref(), Some("0.1.0"));
    assert_eq!(
        updated.previous_checksum.as_deref(),
        Some("sha256:old-checksum")
    );
    assert!(updated.enabled, "enabled state should be preserved");
    assert!(
        updated.checksum.is_some(),
        "new checksum should be recorded"
    );
    assert_ne!(updated.checksum.as_deref(), Some("sha256:old-checksum"));
}

#[test]
fn test_installer_update_preserves_previous_artifact_on_manifest_failure() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "update-failure", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), false)
        .expect("install should succeed");

    fs::write(
        module_path.join("manifest.json"),
        r#"{"id":"update-failure","version":"0.2.0"}"#,
    )
    .expect("write invalid update manifest");

    let error = installer
        .update(Some("update-failure"), false)
        .expect_err("update should fail for invalid manifest");
    let error_message = format!("{error:#}");
    assert!(
        error_message.contains("Old version (0.1.0) remains active"),
        "error should preserve previous version context: {error_message}"
    );

    let lock = ModulesLock::load(&home.join("modules.lock")).expect("load modules.lock");
    let record = lock
        .modules
        .iter()
        .find(|record| record.id == "update-failure")
        .expect("record should exist after failed update");
    assert_eq!(record.version, "0.1.0");
    assert!(
        record.previous_version.is_none(),
        "rollback metadata should not be mutated on failed update"
    );

    let installed_manifest = load_manifest(
        &home
            .join("modules")
            .join("installed")
            .join("update-failure")
            .join("manifest.json"),
    )
    .expect("existing installed manifest should remain valid");
    assert_eq!(installed_manifest.version, "0.1.0");

    let quarantine_entries = fs::read_dir(home.join("modules").join("quarantine"))
        .expect("read quarantine directory")
        .collect::<std::io::Result<Vec<_>>>()
        .expect("collect quarantine entries");
    assert!(
        !quarantine_entries.is_empty(),
        "failed update candidate should be quarantined"
    );
}

#[test]
fn test_installer_update_all_reports_partial_failure() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_a_path = create_test_module(source_root.path(), "module-a", "0.1.0");
    let module_b_path = create_test_module(source_root.path(), "module-b", "0.1.0");

    installer
        .install(module_a_path.to_str().unwrap(), false)
        .expect("module-a install should succeed");
    installer
        .install(module_b_path.to_str().unwrap(), false)
        .expect("module-b install should succeed");

    create_test_module(source_root.path(), "module-a", "0.2.0");
    fs::write(
        module_b_path.join("manifest.json"),
        r#"{"id":"module-b","version":"0.2.0"}"#,
    )
    .expect("write invalid module-b update manifest");

    let error = installer
        .update(None, true)
        .expect_err("update --all should report partial failure");
    let error_message = format!("{error:#}");
    assert!(
        error_message.contains("module(s) failed to update"),
        "expected failure summary in error: {error_message}"
    );
    assert!(
        error_message.contains("module-b"),
        "expected failing module identifier in error: {error_message}"
    );

    let lock = ModulesLock::load(&home.join("modules.lock")).expect("load modules.lock");
    let module_a = lock
        .modules
        .iter()
        .find(|record| record.id == "module-a")
        .expect("module-a record should exist");
    let module_b = lock
        .modules
        .iter()
        .find(|record| record.id == "module-b")
        .expect("module-b record should exist");

    assert_eq!(module_a.version, "0.2.0");
    assert_eq!(module_a.previous_version.as_deref(), Some("0.1.0"));
    assert_eq!(module_b.version, "0.1.0");
}

#[test]
fn test_installer_doctor_reports_healthy_state() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "doctor-healthy", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), true)
        .expect("install should succeed");

    let installed_path = home
        .join("modules")
        .join("installed")
        .join("doctor-healthy");
    let checksum = format!(
        "sha256:{}",
        compute_artifact_sha256(&installed_path).expect("compute checksum")
    );

    let lock_path = home.join("modules.lock");
    let mut lock = ModulesLock::load(&lock_path).expect("load modules.lock");
    let record = lock
        .modules
        .iter_mut()
        .find(|record| record.id == "doctor-healthy")
        .expect("record should exist");
    record.checksum = Some(checksum);
    lock.save(&lock_path).expect("save modules.lock");

    let report = installer.doctor().expect("doctor should succeed");
    assert!(!report.has_errors(), "healthy setup should have no errors");
    assert_eq!(report.error_count(), 0);
    assert_eq!(report.warning_count(), 0);

    let output = report.render();
    assert!(output.contains("[HEALTH] doctor-healthy"));
    assert!(output.contains("[DEPENDENCIES] doctor-healthy"));
    assert!(output.contains("[VERIFICATION] doctor-healthy"));
    assert!(output.contains("[QUARANTINE] doctor-healthy"));
}

#[test]
fn test_installer_doctor_reports_degraded_state() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    let source_root = TempDir::new().expect("failed to create source tempdir");
    let module_path = create_test_module(source_root.path(), "doctor-degraded", "0.1.0");
    installer
        .install(module_path.to_str().unwrap(), false)
        .expect("install should succeed");

    fs::remove_file(
        home.join("modules")
            .join("installed")
            .join("doctor-degraded")
            .join("manifest.json"),
    )
    .expect("remove manifest from installed artifact");

    let lock_path = home.join("modules.lock");
    let mut lock = ModulesLock::load(&lock_path).expect("load modules.lock");
    let record = lock
        .modules
        .iter_mut()
        .find(|record| record.id == "doctor-degraded")
        .expect("record should exist");
    record.quarantine.state = QuarantineState::Quarantined;
    record.quarantine.reason = Some("verification_failed".to_string());
    record.quarantine.since = Some("2026-03-15T10:30:00Z".to_string());
    lock.save(&lock_path).expect("save modules.lock");

    let report = installer
        .doctor()
        .expect("doctor should still return report");
    assert!(report.has_errors(), "degraded setup should report errors");
    assert!(report.error_count() > 0);

    let output = report.render();
    assert!(output.contains("[HEALTH] doctor-degraded ERR"));
    assert!(output.contains("manifest.json missing"));
    assert!(output.contains("[QUARANTINE] doctor-degraded ERR"));
}

#[test]
fn test_installer_doctor_fails_closed_on_unreadable_lockfile() {
    let (_temp, home) = create_test_home();
    fs::write(home.join("modules.lock"), "{ invalid json").expect("write malformed modules.lock");

    let installer = ModuleInstaller::new(home);
    let error = installer
        .doctor()
        .expect_err("doctor should fail closed on malformed lockfile");
    assert!(
        format!("{error:#}").contains("cannot read modules.lock"),
        "error should include lockfile read context"
    );
}

#[test]
fn test_installer_remove_non_existent() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Try to remove non-existent module
    let result = installer.remove("non-existent");
    assert!(
        result.is_err(),
        "remove should fail for non-existent module"
    );
}

#[test]
fn test_installer_invalid_manifest_rejected() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create invalid module (missing required fields)
    let test_module_dir = PathBuf::from("/tmp/test-invalid-module");
    fs::create_dir_all(&test_module_dir).unwrap();

    let invalid_manifest = r#"{
        "id": "invalid",
        "version": "0.1.0"
    }"#;

    fs::write(test_module_dir.join("manifest.json"), invalid_manifest).unwrap();

    // Try to install - should fail validation
    let result = installer.install(test_module_dir.to_str().unwrap(), false);
    assert!(result.is_err(), "install should fail for invalid manifest");

    // Verify nothing was installed
    let lock_path = home.join("modules.lock");
    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 0);

    // Cleanup
    fs::remove_dir_all(&test_module_dir).ok();
}

#[test]
#[cfg(unix)]
fn test_installer_install_tar_archive() {
    use std::process::Command;

    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create test module directory structure
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let module_root = temp_dir.path().join("test-tar-module");
    fs::create_dir_all(&module_root).unwrap();

    // Write manifest.json
    let manifest = r#"{
        "id": "test-tar-module",
        "name": "test-tar-module",
        "version": "0.1.0",
        "kind": "runtime",
        "engine": { "redhorse": ">=0.1.0 <0.2.0" },
        "artifact": { "kind": "bundled", "entry": "modules/runtimes/test-tar-module" },
        "execution": { "mode": "in_process" },
        "trust": { "required": "official" },
        "capabilities": { "requested": [], "parameterized": [] },
        "dependencies": [],
        "config": { "schema": {"type": "object"}, "defaultFragment": {} },
        "activation": { "events": ["startup"], "safeModeEligible": true },
        "install": { "source": "bundled" }
    }"#;
    fs::write(module_root.join("manifest.json"), manifest).unwrap();

    // Create tar archive
    let archive_path = temp_dir.path().join("test-tar-module.tar");
    let status = Command::new("tar")
        .arg("-cf")
        .arg(&archive_path)
        .arg("-C")
        .arg(temp_dir.path())
        .arg("test-tar-module")
        .status()
        .expect("failed to spawn tar");
    assert!(status.success(), "tar creation should succeed");

    // Install from archive
    let result = installer.install(archive_path.to_str().unwrap(), false);
    assert!(
        result.is_ok(),
        "install from tar should succeed: {:?}",
        result.err()
    );
    let module_id = result.unwrap();
    assert_eq!(module_id, "test-tar-module");

    // Verify module is in modules.lock
    let lock_path = home.join("modules.lock");
    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 1);
    assert_eq!(lock.modules[0].id, "test-tar-module");
}

#[test]
#[cfg(unix)]
fn test_installer_install_tar_gz_archive() {
    use std::process::Command;

    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create test module directory structure
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let module_root = temp_dir.path().join("test-targz-module");
    fs::create_dir_all(&module_root).unwrap();

    // Write manifest.json
    let manifest = r#"{
        "id": "test-targz-module",
        "name": "test-targz-module",
        "version": "0.2.0",
        "kind": "runtime",
        "engine": { "redhorse": ">=0.1.0 <0.2.0" },
        "artifact": { "kind": "bundled", "entry": "modules/runtimes/test-targz-module" },
        "execution": { "mode": "in_process" },
        "trust": { "required": "official" },
        "capabilities": { "requested": [], "parameterized": [] },
        "dependencies": [],
        "config": { "schema": {"type": "object"}, "defaultFragment": {} },
        "activation": { "events": ["startup"], "safeModeEligible": true },
        "install": { "source": "bundled" }
    }"#;
    fs::write(module_root.join("manifest.json"), manifest).unwrap();

    // Create tar.gz archive
    let archive_path = temp_dir.path().join("test-targz-module.tar.gz");
    let status = Command::new("tar")
        .arg("-czf")
        .arg(&archive_path)
        .arg("-C")
        .arg(temp_dir.path())
        .arg("test-targz-module")
        .status()
        .expect("failed to spawn tar");
    assert!(status.success(), "tar.gz creation should succeed");

    // Install from archive
    let result = installer.install(archive_path.to_str().unwrap(), false);
    assert!(
        result.is_ok(),
        "install from tar.gz should succeed: {:?}",
        result.err()
    );
    let module_id = result.unwrap();
    assert_eq!(module_id, "test-targz-module");

    // Verify module is in modules.lock
    let lock_path = home.join("modules.lock");
    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 1);
    assert_eq!(lock.modules[0].id, "test-targz-module");
}

#[test]
#[cfg(windows)]
fn test_installer_install_zip_archive() {
    use std::process::Command;

    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create test module directory structure
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let module_root = temp_dir.path().join("test-zip-module");
    fs::create_dir_all(&module_root).unwrap();

    // Write manifest.json
    let manifest = r#"{
        "id": "test-zip-module",
        "name": "test-zip-module",
        "version": "0.3.0",
        "kind": "runtime",
        "engine": { "redhorse": ">=0.1.0 <0.2.0" },
        "artifact": { "kind": "bundled", "entry": "modules/runtimes/test-zip-module" },
        "execution": { "mode": "in_process" },
        "trust": { "required": "official" },
        "capabilities": { "requested": [], "parameterized": [] },
        "dependencies": [],
        "config": { "schema": {"type": "object"}, "defaultFragment": {} },
        "activation": { "events": ["startup"], "safeModeEligible": true },
        "install": { "source": "bundled" }
    }"#;
    fs::write(module_root.join("manifest.json"), manifest).unwrap();

    // Create zip archive using PowerShell
    let archive_path = temp_dir.path().join("test-zip-module.zip");
    let status = Command::new("powershell")
        .arg("-Command")
        .arg("Compress-Archive")
        .arg("-Path")
        .arg(&module_root)
        .arg("-DestinationPath")
        .arg(&archive_path)
        .arg("-Force")
        .status()
        .expect("failed to spawn PowerShell");
    assert!(status.success(), "zip creation should succeed");

    // Install from archive
    let result = installer.install(archive_path.to_str().unwrap(), false);
    assert!(
        result.is_ok(),
        "install from zip should succeed: {:?}",
        result.err()
    );
    let module_id = result.unwrap();
    assert_eq!(module_id, "test-zip-module");

    // Verify module is in modules.lock
    let lock_path = home.join("modules.lock");
    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 1);
    assert_eq!(lock.modules[0].id, "test-zip-module");
}

#[test]
fn test_installer_reject_unsupported_archive_format() {
    let (_temp, home) = create_test_home();
    let installer = ModuleInstaller::new(home.clone());

    // Create a fake unsupported archive file
    let temp_dir = TempDir::new().expect("failed to create temp dir");
    let archive_path = temp_dir.path().join("module.rar");
    fs::write(&archive_path, b"fake rar content").unwrap();

    // Try to install - should fail with explicit error
    let result = installer.install(archive_path.to_str().unwrap(), false);
    assert!(
        result.is_err(),
        "install should fail for unsupported archive"
    );
    let error_msg = result.err().unwrap().to_string();
    assert!(
        error_msg.contains("unsupported archive format"),
        "error should mention unsupported format: {}",
        error_msg
    );
    assert!(
        error_msg.contains(".tar") || error_msg.contains(".zip"),
        "error should list supported formats: {}",
        error_msg
    );

    // Verify nothing was installed
    let lock_path = home.join("modules.lock");
    let lock = ModulesLock::load(&lock_path).expect("failed to load modules.lock");
    assert_eq!(lock.modules.len(), 0);
}
