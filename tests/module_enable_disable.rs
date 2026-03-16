//! Module enable/disable tests.

use redclaw::core::config::modules_lock::ModulesLock;
use redclaw::core::contracts::ActivationPhase;
use redclaw::core::contracts::ModuleInstallRecord;
use redclaw::core::lifecycle::{LoaderConfig, ModuleHost};
use redclaw::core::registry::ModuleRegistry;
use std::fs;

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
fn disabled_module_is_skipped_by_activation_path() {
    let temp = tempfile::tempdir().expect("tempdir");
    let module_dir = temp.path().join("src/modules/runtimes/runtime-native");
    fs::create_dir_all(&module_dir).expect("create module dir");
    fs::write(
        module_dir.join("manifest.json"),
        canonical_manifest("runtime-native", "runtime", &[]),
    )
    .expect("write manifest");

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
        .expect("runtime result");

    assert!(!result.success);
    assert_eq!(result.phase, ActivationPhase::InstallStateResolved);
}

#[test]
fn dependent_module_fails_when_required_dependency_is_disabled() {
    let temp = tempfile::tempdir().expect("tempdir");

    let runtime_dir = temp.path().join("src/modules/runtimes/runtime-native");
    fs::create_dir_all(&runtime_dir).expect("create runtime dir");
    fs::write(
        runtime_dir.join("manifest.json"),
        canonical_manifest("runtime-native", "runtime", &[]),
    )
    .expect("write runtime manifest");

    let channel_dir = temp.path().join("src/modules/channels/channel-cli");
    fs::create_dir_all(&channel_dir).expect("create channel dir");
    fs::write(
        channel_dir.join("manifest.json"),
        canonical_manifest("channel-cli", "channel", &["runtime-native"]),
    )
    .expect("write channel manifest");

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
