//! Manifest validation tests.

use redclaw::core::contracts::{
    ArtifactKind, ExecutionMode, InstallSource, ManifestActivation, ManifestArtifact,
    ManifestCapabilities, ManifestConfig, ManifestEngine, ManifestExecution, ManifestInstall,
    ManifestTrust, ModuleKind, ModuleManifest, TrustRequirement,
};
use redclaw::core::lifecycle::{load_manifest, validate_manifest};
use std::fs;

#[test]
fn invalid_manifest_is_rejected() {
    let invalid = ModuleManifest {
        id: String::new(),
        name: String::new(),
        version: String::new(),
        kind: ModuleKind::Runtime,
        engine: ManifestEngine {
            redhorse: ">=0.1.0 <0.2.0".to_string(),
        },
        artifact: ManifestArtifact {
            kind: ArtifactKind::Bundled,
            entry: "modules/runtimes/runtime-native".to_string(),
        },
        execution: ManifestExecution {
            mode: ExecutionMode::InProcess,
        },
        trust: ManifestTrust {
            required: TrustRequirement::Official,
        },
        capabilities: ManifestCapabilities {
            requested: vec![],
            parameterized: vec![],
        },
        dependencies: vec![],
        config: ManifestConfig {
            schema: serde_json::json!({"type": "object"}),
            default_fragment: serde_json::json!({}),
        },
        activation: ManifestActivation {
            events: vec!["startup".to_string()],
            safe_mode_eligible: true,
        },
        install: ManifestInstall {
            source: InstallSource::Bundled,
        },
    };

    let result = validate_manifest(&invalid);
    assert!(!result.valid);
    assert!(result.errors.iter().any(|e| e.contains("Module ID")));
}

#[test]
fn malformed_manifest_file_fails_closed() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(&manifest_path, "{ not-valid-json }").expect("write malformed manifest");

    let err = load_manifest(&manifest_path).expect_err("malformed manifest must fail");
    let msg = format!("{err:#}");
    assert!(!msg.trim().is_empty(), "error message should be populated");
}

#[test]
fn manifest_with_unknown_field_is_rejected_fail_closed() {
    let temp = tempfile::tempdir().expect("tempdir");
    let manifest_path = temp.path().join("manifest.json");
    fs::write(
        &manifest_path,
        r#"{
            "id": "runtime-native",
            "name": "Native Runtime",
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
            "install": { "source": "bundled" },
            "unknownField": true
        }"#,
    )
    .expect("write manifest");

    let err = load_manifest(&manifest_path).expect_err("unknown fields must fail closed");
    let msg = format!("{err:#}");
    assert!(
        msg.contains("unknown field"),
        "expected unknown field validation error, got: {msg}"
    );
}

#[test]
fn invalid_semver_and_engine_range_are_rejected() {
    let invalid = ModuleManifest {
        id: "runtime-native".to_string(),
        name: "Native Runtime".to_string(),
        version: "not-semver".to_string(),
        kind: ModuleKind::Runtime,
        engine: ManifestEngine {
            redhorse: "not-a-range".to_string(),
        },
        artifact: ManifestArtifact {
            kind: ArtifactKind::Bundled,
            entry: "modules/runtimes/runtime-native".to_string(),
        },
        execution: ManifestExecution {
            mode: ExecutionMode::InProcess,
        },
        trust: ManifestTrust {
            required: TrustRequirement::Official,
        },
        capabilities: ManifestCapabilities {
            requested: vec![],
            parameterized: vec![],
        },
        dependencies: vec![],
        config: ManifestConfig {
            schema: serde_json::json!({"type": "object"}),
            default_fragment: serde_json::json!({}),
        },
        activation: ManifestActivation {
            events: vec!["startup".to_string()],
            safe_mode_eligible: true,
        },
        install: ManifestInstall {
            source: InstallSource::Bundled,
        },
    };

    let result = validate_manifest(&invalid);
    assert!(!result.valid);
    assert!(
        result.errors[0].contains("semver"),
        "expected semver validation failure, got: {:?}",
        result.errors
    );
}

#[test]
fn bundled_manifest_requires_logical_module_entry() {
    let invalid = ModuleManifest {
        id: "runtime-native".to_string(),
        name: "Native Runtime".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Runtime,
        engine: ManifestEngine {
            redhorse: ">=0.1.0 <0.2.0".to_string(),
        },
        artifact: ManifestArtifact {
            kind: ArtifactKind::Bundled,
            entry: "runtime_impl.rs".to_string(),
        },
        execution: ManifestExecution {
            mode: ExecutionMode::InProcess,
        },
        trust: ManifestTrust {
            required: TrustRequirement::Official,
        },
        capabilities: ManifestCapabilities {
            requested: vec![],
            parameterized: vec![],
        },
        dependencies: vec![],
        config: ManifestConfig {
            schema: serde_json::json!({"type": "object"}),
            default_fragment: serde_json::json!({}),
        },
        activation: ManifestActivation {
            events: vec!["startup".to_string()],
            safe_mode_eligible: true,
        },
        install: ManifestInstall {
            source: InstallSource::Bundled,
        },
    };

    let result = validate_manifest(&invalid);
    assert!(!result.valid);
    assert!(
        result.errors[0].contains("bundled artifact.entry"),
        "expected logical entry validation failure, got: {:?}",
        result.errors
    );
}

#[test]
fn config_schema_and_default_fragment_must_be_objects() {
    let invalid = ModuleManifest {
        id: "runtime-native".to_string(),
        name: "Native Runtime".to_string(),
        version: "0.1.0".to_string(),
        kind: ModuleKind::Runtime,
        engine: ManifestEngine {
            redhorse: ">=0.1.0 <0.2.0".to_string(),
        },
        artifact: ManifestArtifact {
            kind: ArtifactKind::Bundled,
            entry: "modules/runtimes/runtime-native".to_string(),
        },
        execution: ManifestExecution {
            mode: ExecutionMode::InProcess,
        },
        trust: ManifestTrust {
            required: TrustRequirement::Official,
        },
        capabilities: ManifestCapabilities {
            requested: vec![],
            parameterized: vec![],
        },
        dependencies: vec![],
        config: ManifestConfig {
            schema: serde_json::json!(true),
            default_fragment: serde_json::json!([]),
        },
        activation: ManifestActivation {
            events: vec!["startup".to_string()],
            safe_mode_eligible: true,
        },
        install: ManifestInstall {
            source: InstallSource::Bundled,
        },
    };

    let result = validate_manifest(&invalid);
    assert!(!result.valid);
    assert!(
        result.errors[0].contains("config.schema"),
        "expected config schema validation failure, got: {:?}",
        result.errors
    );
}
