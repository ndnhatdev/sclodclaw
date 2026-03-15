//! Activation flow tests.

use redclaw::core::config::modules_lock::ModulesLock;
use redclaw::core::contracts::{ActivationPhase, ModuleInstallRecord, QuarantineState};
use redclaw::core::lifecycle::{LoaderConfig, ModuleHost};
use redclaw::core::registry::ModuleRegistry;
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
fn activation_flow_respects_dependencies_and_reaches_activated_phase() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_manifest(
        &temp.path().join("src/modules/runtimes/runtime-native"),
        &canonical_manifest("runtime-native", "runtime", "in_process", &[]),
    );

    write_manifest(
        &temp.path().join("src/modules/channels/channel-cli"),
        &canonical_manifest("channel-cli", "channel", "in_process", &["runtime-native"]),
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
    host.start().expect("host start");

    let runtime = host
        .activation_results()
        .iter()
        .find(|r| r.module_id == "runtime-native")
        .expect("runtime result");
    let channel = host
        .activation_results()
        .iter()
        .find(|r| r.module_id == "channel-cli")
        .expect("channel result");

    assert!(runtime.success);
    assert!(channel.success);
    assert_eq!(runtime.phase, ActivationPhase::Activated);
    assert_eq!(channel.phase, ActivationPhase::Activated);
    assert!(
        channel
            .diagnostics
            .iter()
            .any(|d| d == "dependency_validated"),
        "channel activation should include dependency_validated phase"
    );
}

#[test]
fn quarantined_module_does_not_activate() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_manifest(
        &temp.path().join("src/modules/runtimes/runtime-native"),
        &canonical_manifest("runtime-native", "runtime", "in_process", &[]),
    );

    let lock_path = temp.path().join("state/modules.lock");
    let mut lock = ModulesLock::new();
    lock.add_module(
        ModuleInstallRecord::bundled_v1("runtime-native", "0.1.0", true)
            .with_quarantine_state(QuarantineState::Quarantined),
    );
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

    let runtime = host
        .activation_results()
        .iter()
        .find(|r| r.module_id == "runtime-native")
        .expect("runtime result");

    assert!(!runtime.success);
    assert_eq!(runtime.phase, ActivationPhase::SecurityValidated);
}
