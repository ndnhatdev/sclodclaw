//! Module discovery tests.

use redclaw::core::lifecycle::discover_modules;
use std::fs;

#[test]
fn discovery_finds_nested_module_manifests() {
    let temp = tempfile::tempdir().expect("tempdir");
    let module_dir = temp.path().join("src/modules/runtimes/runtime-native");
    fs::create_dir_all(&module_dir).expect("create module dir");
    fs::write(
        module_dir.join("manifest.json"),
        r#"{
            "id": "runtime-native",
            "name": "runtime-native",
            "version": "0.1.0",
            "kind": "runtime",
            "engine": { "redhorse": ">=0.1.0 <0.2.0" },
            "artifact": { "kind": "bundled", "entry": "modules/runtimes/runtime-native" },
            "execution": { "mode": "in_process" },
            "trust": { "required": "official" },
            "capabilities": { "requested": [], "parameterized": [] },
            "dependencies": [],
            "config": { "schema": {"type": "object"}, "defaultFragment": {} },
            "activation": { "events": ["startup"], "safeModeEligible": true },
            "install": { "source": "bundled" }
        }"#,
    )
    .expect("write manifest");

    let candidates = discover_modules(&temp.path().join("src/modules"));
    assert_eq!(candidates.len(), 1);
    assert!(candidates[0].manifest_path.ends_with("manifest.json"));
}
