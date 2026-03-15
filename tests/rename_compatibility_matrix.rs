//! Evidence tests for rename compatibility matrix coverage.

use std::collections::HashMap;
use std::sync::Mutex;

use redclaw::config::legacy_env::LegacyEnvInput;
use redclaw::config::legacy_paths::{resolve_runtime_workspace_dirs, RuntimeSource};
use redclaw::config::state_paths::{default_config_dir, resolve_config_dir_for_workspace};
use redclaw::core::config::modules_lock::ModulesLock;
use redclaw::Config;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> std::sync::MutexGuard<'static, ()> {
    ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

struct EnvSnapshot {
    previous: HashMap<&'static str, Option<String>>,
}

impl EnvSnapshot {
    fn set(vars: &[(&'static str, Option<String>)]) -> Self {
        let mut previous = HashMap::new();
        for (name, next_value) in vars {
            previous.insert(*name, std::env::var(name).ok());
            match next_value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
        Self { previous }
    }
}

impl Drop for EnvSnapshot {
    fn drop(&mut self) {
        for (name, previous_value) in &self.previous {
            match previous_value {
                Some(value) => std::env::set_var(name, value),
                None => std::env::remove_var(name),
            }
        }
    }
}

struct MarkerSnapshot {
    path: std::path::PathBuf,
    previous: Option<String>,
}

impl MarkerSnapshot {
    fn capture(path: std::path::PathBuf) -> Self {
        let previous = std::fs::read_to_string(&path).ok();
        Self { path, previous }
    }
}

impl Drop for MarkerSnapshot {
    fn drop(&mut self) {
        match &self.previous {
            Some(contents) => {
                if let Some(parent) = self.path.parent() {
                    let _ = std::fs::create_dir_all(parent);
                }
                let _ = std::fs::write(&self.path, contents);
            }
            None => {
                let _ = std::fs::remove_file(&self.path);
            }
        }
    }
}

#[test]
fn legacy_env_collect_prefers_redhorse_values_and_trims_whitespace() {
    let _guard = env_lock();
    let _snapshot = EnvSnapshot::set(&[
        (
            "REDHORSE_CONFIG_DIR",
            Some("  /tmp/redhorse-config  ".to_string()),
        ),
        (
            "REDCLAW_CONFIG_DIR",
            Some("/tmp/redclaw-config".to_string()),
        ),
        (
            "REDHORSE_WORKSPACE",
            Some("  /tmp/redhorse-workspace ".to_string()),
        ),
        (
            "REDCLAW_WORKSPACE",
            Some("/tmp/redclaw-workspace".to_string()),
        ),
    ]);

    let collected = LegacyEnvInput::collect();

    assert_eq!(
        collected.preferred_config_dir().as_deref(),
        Some("/tmp/redhorse-config")
    );
    assert_eq!(
        collected.preferred_workspace_dir().as_deref(),
        Some("/tmp/redhorse-workspace")
    );
}

#[tokio::test]
async fn runtime_dirs_accept_redhorse_config_dir_env_input() {
    let _guard = env_lock();
    let temp = tempfile::tempdir().expect("tempdir");
    let redhorse_config = temp.path().join("redhorse-config");

    let _snapshot = EnvSnapshot::set(&[
        (
            "REDHORSE_CONFIG_DIR",
            Some(redhorse_config.to_string_lossy().into_owned()),
        ),
        ("REDCLAW_CONFIG_DIR", None),
        ("REDHORSE_WORKSPACE", None),
        ("REDCLAW_WORKSPACE", None),
    ]);

    let (config_dir, workspace_dir, source) = resolve_runtime_workspace_dirs()
        .await
        .expect("resolve runtime dirs");

    assert_eq!(source, RuntimeSource::RedhorseConfigDirEnv);
    assert_eq!(config_dir, redhorse_config);
    assert_eq!(workspace_dir, config_dir.join("workspace"));
}

#[tokio::test]
async fn runtime_dirs_accept_redclaw_config_dir_env_when_redhorse_absent() {
    let _guard = env_lock();
    let temp = tempfile::tempdir().expect("tempdir");
    let redclaw_config = temp.path().join("redclaw-config");

    let _snapshot = EnvSnapshot::set(&[
        ("REDHORSE_CONFIG_DIR", None),
        (
            "REDCLAW_CONFIG_DIR",
            Some(redclaw_config.to_string_lossy().into_owned()),
        ),
        ("REDHORSE_WORKSPACE", None),
        ("REDCLAW_WORKSPACE", None),
    ]);

    let (config_dir, workspace_dir, source) = resolve_runtime_workspace_dirs()
        .await
        .expect("resolve runtime dirs");

    assert_eq!(source, RuntimeSource::ZeroclawConfigDirEnv);
    assert_eq!(config_dir, redclaw_config);
    assert_eq!(workspace_dir, config_dir.join("workspace"));
}

#[tokio::test]
async fn runtime_dirs_accept_redhorse_workspace_env_when_redclaw_absent() {
    let _guard = env_lock();
    let temp = tempfile::tempdir().expect("tempdir");
    let redhorse_workspace = temp.path().join("legacy-workspace-root");

    let _snapshot = EnvSnapshot::set(&[
        ("REDHORSE_CONFIG_DIR", None),
        ("REDCLAW_CONFIG_DIR", None),
        (
            "REDHORSE_WORKSPACE",
            Some(redhorse_workspace.to_string_lossy().into_owned()),
        ),
        ("REDCLAW_WORKSPACE", None),
    ]);

    let (config_dir, workspace_dir, source) = resolve_runtime_workspace_dirs()
        .await
        .expect("resolve runtime dirs");

    assert_eq!(source, RuntimeSource::RedhorseWorkspaceEnv);
    assert_eq!(config_dir, redhorse_workspace);
    assert_eq!(workspace_dir, config_dir.join("workspace"));
}

#[tokio::test]
async fn runtime_dirs_accept_redclaw_workspace_env_when_redhorse_absent() {
    let _guard = env_lock();
    let temp = tempfile::tempdir().expect("tempdir");
    let redclaw_workspace = temp.path().join("legacy-workspace-root");

    let _snapshot = EnvSnapshot::set(&[
        ("REDHORSE_CONFIG_DIR", None),
        ("REDCLAW_CONFIG_DIR", None),
        ("REDHORSE_WORKSPACE", None),
        (
            "REDCLAW_WORKSPACE",
            Some(redclaw_workspace.to_string_lossy().into_owned()),
        ),
    ]);

    let (config_dir, workspace_dir, source) = resolve_runtime_workspace_dirs()
        .await
        .expect("resolve runtime dirs");

    assert_eq!(source, RuntimeSource::ZeroclawWorkspaceEnv);
    assert_eq!(config_dir, redclaw_workspace);
    assert_eq!(workspace_dir, config_dir.join("workspace"));
}

#[test]
fn workspace_resolution_prefers_workspace_config_when_present() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace_dir = temp.path().join("profiles").join("gamma");

    std::fs::create_dir_all(&workspace_dir).expect("create workspace dir");
    std::fs::write(
        workspace_dir.join("config.toml"),
        "default_temperature = 0.7\n",
    )
    .expect("write workspace config");

    let (resolved_config_dir, resolved_workspace_dir) =
        resolve_config_dir_for_workspace(&workspace_dir);

    assert_eq!(resolved_config_dir, workspace_dir);
    assert_eq!(resolved_workspace_dir, workspace_dir.join("workspace"));
}

#[test]
fn workspace_resolution_uses_legacy_home_config_when_present() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace_dir = temp.path().join("profiles").join("alpha");
    let legacy_home = workspace_dir
        .parent()
        .expect("workspace has parent")
        .join(".redhorse");

    std::fs::create_dir_all(&legacy_home).expect("create legacy home dir");
    std::fs::write(
        legacy_home.join("config.toml"),
        "default_temperature = 0.7\n",
    )
    .expect("write legacy config");

    let (resolved_config_dir, resolved_workspace_dir) =
        resolve_config_dir_for_workspace(&workspace_dir);

    assert_eq!(resolved_config_dir, legacy_home);
    assert_eq!(resolved_workspace_dir, workspace_dir);
}

#[tokio::test]
async fn runtime_dirs_use_active_workspace_marker_when_env_missing() {
    let _guard = env_lock();
    let temp_home = tempfile::tempdir().expect("tempdir");
    let default_config_dir = default_config_dir().expect("resolve default config dir");
    let marker_path = default_config_dir.join("active_workspace.toml");
    let marker_config_dir = temp_home.path().join("profiles").join("agent-alpha");
    let _marker_snapshot = MarkerSnapshot::capture(marker_path.clone());

    std::fs::create_dir_all(&default_config_dir).expect("create default config dir");
    std::fs::create_dir_all(&marker_config_dir).expect("create marker config dir");
    std::fs::write(
        &marker_path,
        format!("config_dir = \"{}\"\n", marker_config_dir.display()),
    )
    .expect("write active workspace marker");

    let _snapshot = EnvSnapshot::set(&[
        ("REDHORSE_CONFIG_DIR", None),
        ("REDCLAW_CONFIG_DIR", None),
        ("REDHORSE_WORKSPACE", None),
        ("REDCLAW_WORKSPACE", None),
    ]);

    let (config_dir, workspace_dir, source) = resolve_runtime_workspace_dirs()
        .await
        .expect("resolve runtime dirs");

    assert_eq!(source, RuntimeSource::ActiveWorkspaceMarker);
    assert_eq!(config_dir, marker_config_dir);
    assert_eq!(workspace_dir, config_dir.join("workspace"));
}

#[test]
fn config_file_alias_dburl_still_deserializes_to_db_url() {
    let raw = r#"
default_temperature = 0.7

[storage.provider.config]
provider = "postgres"
dbURL = "postgres://postgres:postgres@localhost:5432/redhorse"
"#;

    let parsed: Config = toml::from_str(raw).expect("parse config");
    assert_eq!(parsed.storage.provider.config.provider, "postgres");
    assert_eq!(
        parsed.storage.provider.config.db_url.as_deref(),
        Some("postgres://postgres:postgres@localhost:5432/redhorse")
    );
}

#[test]
fn modules_lock_legacy_field_aliases_load_and_rewrite_to_canonical_keys() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");
    std::fs::create_dir_all(
        lock_path
            .parent()
            .expect("lock path should include parent directory"),
    )
    .expect("create lock parent");

    let legacy_json = r#"{
  "schema_version": 1,
  "modules": [
    {
      "id": "provider-openai-compatible",
      "version": "0.1.0",
      "artifact": { "kind": "bundled", "entry": "modules/providers/provider-openai-compatible" },
      "install": { "source": "bundled" },
      "trust": { "tier": "official", "verification_state": "verified" },
      "source": { "uri": null, "path": null },
      "execution": { "resolved_mode": "process" },
      "quarantine": { "state": "clear", "reason": null, "since": null },
      "enabled": true,
      "checksum": null,
      "signature": null,
      "previous_version": null,
      "previous_checksum": null
    }
  ]
}"#;

    std::fs::write(&lock_path, legacy_json).expect("write legacy lockfile");

    let loaded = ModulesLock::load(&lock_path).expect("legacy lockfile should load");
    assert_eq!(loaded.schema_version, ModulesLock::CURRENT_SCHEMA);
    assert_eq!(loaded.modules.len(), 1);

    loaded
        .save_atomic(&lock_path)
        .expect("rewriting lockfile should succeed");
    let rewritten = std::fs::read_to_string(&lock_path).expect("read rewritten lockfile");

    assert!(rewritten.contains("\"schemaVersion\""));
    assert!(rewritten.contains("\"verificationState\""));
    assert!(rewritten.contains("\"resolvedMode\""));
    assert!(rewritten.contains("\"previousVersion\""));
    assert!(rewritten.contains("\"previousChecksum\""));
    assert!(!rewritten.contains("\"schema_version\""));
    assert!(!rewritten.contains("\"verification_state\""));
    assert!(!rewritten.contains("\"resolved_mode\""));
    assert!(!rewritten.contains("\"previous_version\""));
    assert!(!rewritten.contains("\"previous_checksum\""));
}

#[test]
fn modules_lock_unknown_fields_fail_closed() {
    let temp = tempfile::tempdir().expect("tempdir");
    let lock_path = temp.path().join("state/modules.lock");
    std::fs::create_dir_all(
        lock_path
            .parent()
            .expect("lock path should include parent directory"),
    )
    .expect("create lock parent");

    let with_unknown_field = r#"{
  "schemaVersion": 1,
  "modules": [
    {
      "id": "provider-openai-compatible",
      "version": "0.1.0",
      "artifact": { "kind": "bundled", "entry": "modules/providers/provider-openai-compatible" },
      "install": { "source": "bundled" },
      "trust": { "tier": "official", "verificationState": "verified" },
      "source": { "uri": null, "path": null },
      "execution": { "resolvedMode": "process" },
      "quarantine": { "state": "clear", "reason": null, "since": null },
      "enabled": true,
      "checksum": null,
      "signature": null,
      "previousVersion": null,
      "previousChecksum": null,
      "unknown_field": true
    }
  ]
}"#;

    std::fs::write(&lock_path, with_unknown_field).expect("write lockfile");
    let err = ModulesLock::load(&lock_path).expect_err("unknown fields must fail");
    let msg = format!("{err:#}");
    assert!(msg.contains("unknown field"));
}

#[test]
fn unsupported_legacy_zero_prefix_env_vars_are_ignored() {
    let _guard = env_lock();
    let _snapshot = EnvSnapshot::set(&[
        ("ZEROCLAW_CONFIG_DIR", Some("/tmp/zero-config".to_string())),
        (
            "ZEROCLAW_WORKSPACE",
            Some("/tmp/zero-workspace".to_string()),
        ),
        ("REDHORSE_CONFIG_DIR", None),
        ("REDCLAW_CONFIG_DIR", None),
        ("REDHORSE_WORKSPACE", None),
        ("REDCLAW_WORKSPACE", None),
    ]);

    let collected = LegacyEnvInput::collect();
    assert_eq!(collected.preferred_config_dir(), None);
    assert_eq!(collected.preferred_workspace_dir(), None);
}

#[test]
fn unsupported_primary_redclaw_home_name_is_not_auto_detected_for_workspace() {
    let temp = tempfile::tempdir().expect("tempdir");
    let workspace_dir = temp.path().join("profiles").join("beta");
    let unsupported_redclaw_home = workspace_dir
        .parent()
        .expect("workspace has parent")
        .join(".redclaw");

    std::fs::create_dir_all(&unsupported_redclaw_home).expect("create unsupported home dir");
    std::fs::write(
        unsupported_redclaw_home.join("config.toml"),
        "default_temperature = 0.7\n",
    )
    .expect("write unsupported config");

    let (resolved_config_dir, resolved_workspace_dir) =
        resolve_config_dir_for_workspace(&workspace_dir);

    assert_eq!(resolved_config_dir, workspace_dir);
    assert_eq!(resolved_workspace_dir, workspace_dir.join("workspace"));
}
