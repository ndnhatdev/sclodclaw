//! Vertical slice CLI tests.

use redclaw::core::config::modules_lock::ModulesLock;
use redclaw::core::contracts::{ActivationPhase, ModuleInstallRecord};
use redclaw::core::lifecycle::{LoaderConfig, ModuleHost};
use redclaw::core::registry::ModuleRegistry;
use std::fs;

fn write_manifest(dir: &std::path::Path, manifest: &str) {
    fs::create_dir_all(dir).expect("create module dir");
    fs::write(dir.join("manifest.json"), manifest).expect("write manifest");
}

fn canonical_manifest(
    id: &str,
    kind: &str,
    mode: &str,
    deps: &[&str],
    safe_mode_eligible: bool,
) -> String {
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
            "activation": {{ "events": ["startup"], "safeModeEligible": {safe_mode_eligible} }},
            "install": {{ "source": "bundled" }}
        }}"#
    )
}

#[test]
fn baseline_cli_slice_boots_while_optional_process_modules_remain_disabled() {
    let temp = tempfile::tempdir().expect("tempdir");

    write_manifest(
        &temp.path().join("src/modules/runtimes/runtime-native"),
        &canonical_manifest("runtime-native", "runtime", "in_process", &[], true),
    );
    write_manifest(
        &temp.path().join("src/modules/channels/channel-cli"),
        &canonical_manifest(
            "channel-cli",
            "channel",
            "in_process",
            &["runtime-native"],
            true,
        ),
    );
    write_manifest(
        &temp
            .path()
            .join("src/modules/providers/provider-openai-compatible"),
        &canonical_manifest(
            "provider-openai-compatible",
            "provider",
            "process",
            &["runtime-native"],
            false,
        ),
    );
    write_manifest(
        &temp.path().join("src/modules/tools/tool-shell"),
        &canonical_manifest("tool-shell", "tool", "process", &["runtime-native"], false),
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
    lock.add_module(ModuleInstallRecord::bundled_v1(
        "provider-openai-compatible",
        "0.1.0",
        false,
    ));
    lock.add_module(ModuleInstallRecord::bundled_v1(
        "tool-shell",
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
    let provider = host
        .activation_results()
        .iter()
        .find(|r| r.module_id == "provider-openai-compatible")
        .expect("provider result");
    let tool = host
        .activation_results()
        .iter()
        .find(|r| r.module_id == "tool-shell")
        .expect("tool result");

    assert!(runtime.success);
    assert!(channel.success);
    assert!(!provider.success);
    assert!(!tool.success);
    assert_eq!(provider.phase, ActivationPhase::InstallStateResolved);
    assert_eq!(tool.phase, ActivationPhase::InstallStateResolved);
}
