//! Modules lock schema validation tests.

use redclaw::core::config::modules_lock::ModulesLock;
use redclaw::core::contracts::ModuleInstallRecord;
use redclaw::core::lifecycle::{LoaderConfig, ModuleHost};
use redclaw::core::registry::ModuleRegistry;
use std::fs;

#[test]
fn modules_lock_roundtrip_preserves_install_records() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");

    let mut lock = ModulesLock::new();
    lock.add_module(ModuleInstallRecord::bundled_v1(
        "runtime-native",
        "0.1.0",
        true,
    ));
    lock.save_atomic(&lock_path).expect("save lock");

    let loaded = ModulesLock::load(&lock_path).expect("load lock");
    assert_eq!(loaded.schema_version, ModulesLock::CURRENT_SCHEMA);
    assert_eq!(loaded.modules.len(), 1);
    assert_eq!(loaded.modules[0].id, "runtime-native");
    assert_eq!(
        loaded.modules[0].artifact.entry,
        "modules/runtimes/runtime-native"
    );
    assert_eq!(
        loaded.modules[0].trust.verification_state, "verified",
        "lock should preserve canonical trust metadata"
    );
}

#[test]
fn malformed_lockfile_is_rejected() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");
    fs::create_dir_all(lock_path.parent().unwrap()).expect("create lock dir");
    fs::write(&lock_path, "{ this-is-not-json }").expect("write malformed lock");

    let err = ModulesLock::load(&lock_path).expect_err("malformed lock must fail");
    let msg = format!("{err:#}");
    assert!(!msg.trim().is_empty(), "error message should be populated");
}

#[test]
fn unknown_lock_record_fields_are_rejected_fail_closed() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");
    fs::create_dir_all(lock_path.parent().unwrap()).expect("create lock dir");
    fs::write(
        &lock_path,
        r#"{
            "schema_version": 1,
            "modules": [
                {
                    "id": "runtime-native",
                    "version": "0.1.0",
                    "artifact": { "kind": "bundled", "entry": "modules/runtimes/runtime-native" },
                    "install": { "source": "bundled" },
                    "trust": { "tier": "official", "verification_state": "verified" },
                    "source": { "uri": null, "path": null },
                    "execution": { "resolved_mode": "in_process" },
                    "quarantine": { "state": "clear", "reason": null, "since": null },
                    "enabled": true,
                    "checksum": null,
                    "signature": null,
                    "previous_version": null,
                    "previous_checksum": null,
                    "unknown_field": true
                }
            ]
        }"#,
    )
    .expect("write lock");

    let err = ModulesLock::load(&lock_path).expect_err("unknown fields must fail closed");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("unknown field"),
        "expected unknown field rejection, got: {msg}"
    );
}

#[test]
fn invalid_manifest_does_not_write_partial_lock_record() {
    let temp = tempfile::tempdir().expect("tempdir");
    let module_dir = temp.path().join("src/modules/tools/tool-shell");
    fs::create_dir_all(&module_dir).expect("create module dir");
    fs::write(
        module_dir.join("manifest.json"),
        r#"{
            "id": "",
            "version": "0.1.0",
            "kind": "tool",
            "execution": { "mode": "process" },
            "dependencies": []
        }"#,
    )
    .expect("write invalid manifest");

    let lock_path = temp.path().join("state/modules.lock");
    let mut host = ModuleHost::new(
        ModuleRegistry::new(),
        LoaderConfig {
            search_paths: vec![temp.path().join("src/modules")],
            lock_path: lock_path.clone(),
            validate: true,
        },
    );
    host.start()
        .expect("host start with invalid manifest skipped");

    let loaded = ModulesLock::load(&lock_path).expect("load lock");
    assert!(
        loaded.get_module("").is_none(),
        "invalid install-time manifest must not write partial lock record"
    );
}

#[test]
fn duplicate_module_ids_are_rejected_fail_closed() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");
    fs::create_dir_all(lock_path.parent().unwrap()).expect("create lock dir");
    fs::write(
        &lock_path,
        r#"{
            "schemaVersion": 1,
            "modules": [
                {
                    "id": "runtime-native",
                    "version": "0.1.0",
                    "artifact": { "kind": "bundled", "entry": "modules/runtimes/runtime-native" },
                    "install": { "source": "bundled" },
                    "trust": { "tier": "official", "verificationState": "verified" },
                    "source": { "uri": null, "path": null },
                    "execution": { "resolvedMode": "in_process" },
                    "quarantine": { "state": "clear", "reason": null, "since": null },
                    "enabled": true,
                    "checksum": null,
                    "signature": null,
                    "previousVersion": null,
                    "previousChecksum": null
                },
                {
                    "id": "runtime-native",
                    "version": "0.1.0",
                    "artifact": { "kind": "bundled", "entry": "modules/runtimes/runtime-native" },
                    "install": { "source": "bundled" },
                    "trust": { "tier": "official", "verificationState": "verified" },
                    "source": { "uri": null, "path": null },
                    "execution": { "resolvedMode": "in_process" },
                    "quarantine": { "state": "clear", "reason": null, "since": null },
                    "enabled": true,
                    "checksum": null,
                    "signature": null,
                    "previousVersion": null,
                    "previousChecksum": null
                }
            ]
        }"#,
    )
    .expect("write lock");

    let err = ModulesLock::load(&lock_path).expect_err("duplicate ids must fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("duplicate module id"),
        "expected duplicate id rejection, got: {msg}"
    );
}

#[test]
fn local_dir_record_requires_source_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");
    fs::create_dir_all(lock_path.parent().unwrap()).expect("create lock dir");
    fs::write(
        &lock_path,
        r#"{
            "schemaVersion": 1,
            "modules": [
                {
                    "id": "memory-sqlite",
                    "version": "0.1.0",
                    "artifact": { "kind": "local_dir", "entry": "modules/memory/memory-sqlite" },
                    "install": { "source": "local_dir" },
                    "trust": { "tier": "reviewed", "verificationState": "verified" },
                    "source": { "uri": null, "path": null },
                    "execution": { "resolvedMode": "process" },
                    "quarantine": { "state": "clear", "reason": null, "since": null },
                    "enabled": true,
                    "checksum": null,
                    "signature": null,
                    "previousVersion": null,
                    "previousChecksum": null
                }
            ]
        }"#,
    )
    .expect("write lock");

    let err = ModulesLock::load(&lock_path).expect_err("missing source path must fail");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("source.path"),
        "expected source.path validation failure, got: {msg}"
    );
}
