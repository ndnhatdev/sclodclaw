//! Safe mode boot tests.

use redclaw::core::config::modules_lock::ModulesLock;
use redclaw::core::contracts::ModuleInstallRecord;
use redclaw::core::lifecycle::{
    boot_safe_mode, verify_baseline_modules, LoaderConfig, ModuleHost, SAFE_MODE_BASELINE_MODULES,
};
use redclaw::core::registry::ModuleRegistry;
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
            "engine": {{ "redclaw": ">=0.1.0 <0.2.0" }},
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
fn safe_mode_boot_activates_baseline_via_real_host_path() {
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
    for module_id in SAFE_MODE_BASELINE_MODULES {
        lock.add_module(ModuleInstallRecord::bundled_v1(*module_id, "0.1.0", true));
    }
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
}
