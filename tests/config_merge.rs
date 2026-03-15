//! Config merge boundary tests.

use redclaw::config::Config;
use redclaw::core::config::modules_lock::ModulesLock as CoreModulesLock;
use redclaw::core::contracts::ModuleInstallRecord;

#[test]
fn desired_config_does_not_embed_install_state_fields() {
    let config = Config::default();
    let serialized = toml::to_string(&config).expect("serialize config");

    assert!(
        !serialized.contains("schema_version"),
        "desired config must not embed modules.lock schema version"
    );
    assert!(
        !serialized.contains("quarantine"),
        "desired config must not embed install quarantine state"
    );
}

#[test]
fn install_state_is_authoritative_in_modules_lock() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");

    let mut lock = CoreModulesLock::new();
    lock.add_module(ModuleInstallRecord::bundled_v1(
        "provider-openai-compatible",
        "0.1.0",
        false,
    ));
    lock.save_atomic(&lock_path).expect("save lock");

    let loaded = CoreModulesLock::load(&lock_path).expect("load lock");
    let provider = loaded
        .get_module("provider-openai-compatible")
        .expect("provider lock record");
    assert!(
        !provider.enabled,
        "modules.lock should preserve disabled state"
    );
}
