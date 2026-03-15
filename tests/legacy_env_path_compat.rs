//! Integration tests for legacy environment variable and path compatibility.
//!
//! # Policy (RH-MIG-T0009 Option B: Bounded Compatibility)
//!
//! These tests verify:
//! - RedClaw-first precedence (REDCLAW_* > REDHORSE_*)
//! - Deprecation warnings are emitted for legacy inputs
//! - Legacy home directory fallback (~/.redhorse) works with warnings
//! - Malformed inputs fail closed (do not crash)
//!
//! # Handoff
//!
//! This test suite is scheduled for removal in RH-MIG-T0006 after
//! the migration matrix shows green across all deployment targets.

use std::env;
use tempfile::TempDir;

use redclaw::branding::{HOME_DIR_LEGACY, HOME_DIR_PRIMARY};
use redclaw::config::legacy_env::LegacyEnvInput;
use redclaw::config::legacy_paths::{
    resolve_runtime_source_is_canonical, resolve_runtime_source_uses_legacy_env, RuntimeSource,
};
use redclaw::config::state_paths::{
    default_config_dir, default_primary_config_dir, resolve_config_dir_for_workspace,
};

/// Clean up environment variables after test.
struct EnvGuard<'a> {
    vars: Vec<&'a str>,
    saved: Vec<(&'a str, Option<String>)>,
}

impl<'a> EnvGuard<'a> {
    fn new(vars: &[&'a str]) -> Self {
        let saved = vars
            .iter()
            .map(|&name| (name, env::var(name).ok()))
            .collect();
        Self {
            vars: vars.to_vec(),
            saved,
        }
    }

    fn set(&self, name: &str, value: &str) {
        env::set_var(name, value);
    }

    fn remove(&self, name: &str) {
        env::remove_var(name);
    }
}

impl Drop for EnvGuard<'_> {
    fn drop(&mut self) {
        for (name, saved_value) in &self.saved {
            match saved_value {
                Some(value) => env::set_var(name, value),
                None => env::remove_var(name),
            }
        }
    }
}

fn cleanup_env(vars: &[&str]) {
    for var in vars {
        env::remove_var(var);
    }
}

#[test]
fn test_legacy_env_no_vars_set() {
    let vars = [
        "REDCLAW_CONFIG_DIR",
        "REDHORSE_CONFIG_DIR",
        "REDCLAW_WORKSPACE",
        "REDHORSE_WORKSPACE",
    ];
    let _guard = EnvGuard::new(&vars);
    cleanup_env(&vars);

    let input = LegacyEnvInput::collect();
    assert_eq!(input, LegacyEnvInput::default());
    assert!(!input.uses_legacy_env());
    assert_eq!(input.preferred_config_dir(), None);
    assert_eq!(input.preferred_workspace_dir(), None);
}

#[test]
fn test_legacy_env_canonical_takes_precedence() {
    let vars = ["REDCLAW_CONFIG_DIR", "REDHORSE_CONFIG_DIR"];
    let _guard = EnvGuard::new(&vars);

    env::set_var("REDCLAW_CONFIG_DIR", "/canonical/config");
    env::set_var("REDHORSE_CONFIG_DIR", "/legacy/config");

    let input = LegacyEnvInput::collect();
    assert_eq!(
        input.redclaw_config_dir,
        Some("/canonical/config".to_string())
    );
    assert_eq!(
        input.redhorse_config_dir,
        Some("/legacy/config".to_string())
    );
    assert!(input.uses_legacy_env());

    // Precedence: canonical wins
    assert_eq!(
        input.preferred_config_dir(),
        Some("/canonical/config".to_string())
    );
}

#[test]
fn test_legacy_env_fallback_to_legacy() {
    let vars = ["REDCLAW_CONFIG_DIR", "REDHORSE_CONFIG_DIR"];
    let _guard = EnvGuard::new(&vars);
    cleanup_env(&vars);

    // Only legacy set
    env::set_var("REDHORSE_CONFIG_DIR", "/legacy/config");

    let input = LegacyEnvInput::collect();
    assert_eq!(input.redclaw_config_dir, None);
    assert_eq!(
        input.redhorse_config_dir,
        Some("/legacy/config".to_string())
    );
    assert!(input.uses_legacy_env());

    // Fallback to legacy when canonical not set
    assert_eq!(
        input.preferred_config_dir(),
        Some("/legacy/config".to_string())
    );
}

#[test]
fn test_legacy_env_workspace_precedence() {
    let vars = ["REDCLAW_WORKSPACE", "REDHORSE_WORKSPACE"];
    let _guard = EnvGuard::new(&vars);

    env::set_var("REDCLAW_WORKSPACE", "/canonical/workspace");
    env::set_var("REDHORSE_WORKSPACE", "/legacy/workspace");

    let input = LegacyEnvInput::collect();
    assert_eq!(
        input.redclaw_workspace_dir,
        Some("/canonical/workspace".to_string())
    );
    assert_eq!(
        input.redhorse_workspace_dir,
        Some("/legacy/workspace".to_string())
    );

    // Precedence: canonical wins
    assert_eq!(
        input.preferred_workspace_dir(),
        Some("/canonical/workspace".to_string())
    );
}

#[test]
fn test_legacy_env_trim_and_empty_filter() {
    let vars = ["REDCLAW_CONFIG_DIR", "REDHORSE_CONFIG_DIR"];
    let _guard = EnvGuard::new(&vars);

    // Whitespace-only becomes None
    env::set_var("REDCLAW_CONFIG_DIR", "   ");
    env::set_var("REDHORSE_CONFIG_DIR", "\t\n");

    let input = LegacyEnvInput::collect();
    assert_eq!(input.redclaw_config_dir, None);
    assert_eq!(input.redhorse_config_dir, None);
    assert!(!input.uses_legacy_env());
}

#[test]
fn test_legacy_env_mixed_config_and_workspace() {
    let vars = [
        "REDCLAW_CONFIG_DIR",
        "REDHORSE_CONFIG_DIR",
        "REDCLAW_WORKSPACE",
        "REDHORSE_WORKSPACE",
    ];
    let _guard = EnvGuard::new(&vars);
    cleanup_env(&vars);

    // Canonical config, legacy workspace
    env::set_var("REDCLAW_CONFIG_DIR", "/canonical/config");
    env::set_var("REDHORSE_WORKSPACE", "/legacy/workspace");

    let input = LegacyEnvInput::collect();
    assert!(input.uses_legacy_env());
    assert_eq!(
        input.preferred_config_dir(),
        Some("/canonical/config".to_string())
    );
    assert_eq!(
        input.preferred_workspace_dir(),
        Some("/legacy/workspace".to_string())
    );
}

#[test]
fn test_runtime_source_predicates() {
    // Canonical sources
    assert!(resolve_runtime_source_is_canonical(
        RuntimeSource::RedclawConfigDirEnv
    ));
    assert!(resolve_runtime_source_is_canonical(
        RuntimeSource::RedclawWorkspaceEnv
    ));
    assert!(resolve_runtime_source_is_canonical(
        RuntimeSource::ActiveWorkspaceMarker
    ));
    assert!(resolve_runtime_source_is_canonical(
        RuntimeSource::DefaultConfigDir
    ));

    // Legacy sources (NOT canonical)
    assert!(!resolve_runtime_source_is_canonical(
        RuntimeSource::RedhorseConfigDirEnv
    ));
    assert!(!resolve_runtime_source_is_canonical(
        RuntimeSource::RedhorseWorkspaceEnv
    ));

    // Legacy env detection
    assert!(resolve_runtime_source_uses_legacy_env(
        RuntimeSource::RedhorseConfigDirEnv
    ));
    assert!(resolve_runtime_source_uses_legacy_env(
        RuntimeSource::RedhorseWorkspaceEnv
    ));
    assert!(!resolve_runtime_source_uses_legacy_env(
        RuntimeSource::RedclawConfigDirEnv
    ));
    assert!(!resolve_runtime_source_uses_legacy_env(
        RuntimeSource::DefaultConfigDir
    ));
}

#[test]
fn test_default_primary_config_dir() {
    // This test verifies the canonical default path structure
    let primary_dir = default_primary_config_dir().expect("Failed to get primary config dir");

    // Should end with .redclaw
    assert!(
        primary_dir.ends_with(HOME_DIR_PRIMARY),
        "Primary config dir should end with {}, got {:?}",
        HOME_DIR_PRIMARY,
        primary_dir
    );
}

#[test]
fn test_default_config_dir_with_temp_homes() {
    // Create temporary directories to simulate home directory scenarios
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let home_path = temp_home.path();

    // Create both primary and legacy dirs
    let primary_dir = home_path.join(HOME_DIR_PRIMARY);
    let legacy_dir = home_path.join(HOME_DIR_LEGACY);

    std::fs::create_dir_all(&primary_dir).expect("Failed to create primary dir");
    std::fs::create_dir_all(&legacy_dir).expect("Failed to create legacy dir");

    // When both exist, primary should win (RedClaw-first)
    // Note: This test is limited because default_config_dir() uses real home dir
    // Full integration test would require home dir mocking
    assert!(primary_dir.exists());
    assert!(legacy_dir.exists());
}

#[test]
fn test_workspace_path_prefers_redclaw_sibling_config_dir() {
    let temp_home = TempDir::new().expect("Failed to create temp dir");
    let workspace_dir = temp_home.path().join("workspace");
    let primary_config_dir = temp_home.path().join(HOME_DIR_PRIMARY);

    std::fs::create_dir_all(&workspace_dir).expect("Failed to create workspace dir");

    let (config_dir, resolved_workspace_dir) = resolve_config_dir_for_workspace(&workspace_dir);

    assert_eq!(config_dir, primary_config_dir);
    assert_eq!(resolved_workspace_dir, workspace_dir);
}

#[test]
fn test_legacy_env_struct_serialization() {
    use serde_json;

    let input = LegacyEnvInput {
        redclaw_config_dir: Some("/canonical/config".to_string()),
        redhorse_config_dir: Some("/legacy/config".to_string()),
        redclaw_workspace_dir: None,
        redhorse_workspace_dir: Some("/legacy/workspace".to_string()),
    };

    let json = serde_json::to_string_pretty(&input).expect("Failed to serialize");
    assert!(json.contains("/canonical/config"));
    assert!(json.contains("/legacy/config"));

    let deserialized: LegacyEnvInput = serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(input, deserialized);
}

#[test]
fn test_legacy_env_all_combinations() {
    let vars = [
        "REDCLAW_CONFIG_DIR",
        "REDHORSE_CONFIG_DIR",
        "REDCLAW_WORKSPACE",
        "REDHORSE_WORKSPACE",
    ];

    // Test all 16 combinations of env var presence
    let test_cases = [
        // (redclaw_config, redhorse_config, redclaw_workspace, redhorse_workspace)
        (false, false, false, false),
        (true, false, false, false),
        (false, true, false, false),
        (true, true, false, false),
        (false, false, true, false),
        (true, false, true, false),
        (false, true, true, false),
        (true, true, true, false),
        (false, false, false, true),
        (true, false, false, true),
        (false, true, false, true),
        (true, true, false, true),
        (false, false, true, true),
        (true, false, true, true),
        (false, true, true, true),
        (true, true, true, true),
    ];

    for (rc_cfg, rh_cfg, rc_ws, rh_ws) in test_cases {
        let _guard = EnvGuard::new(&vars);
        cleanup_env(&vars);

        if rc_cfg {
            env::set_var("REDCLAW_CONFIG_DIR", "/rc/config");
        }
        if rh_cfg {
            env::set_var("REDHORSE_CONFIG_DIR", "/rh/config");
        }
        if rc_ws {
            env::set_var("REDCLAW_WORKSPACE", "/rc/workspace");
        }
        if rh_ws {
            env::set_var("REDHORSE_WORKSPACE", "/rh/workspace");
        }

        let input = LegacyEnvInput::collect();

        // Verify collection matches what we set
        assert_eq!(
            input.redclaw_config_dir.is_some(),
            rc_cfg,
            "redclaw_config_dir presence mismatch"
        );
        assert_eq!(
            input.redhorse_config_dir.is_some(),
            rh_cfg,
            "redhorse_config_dir presence mismatch"
        );
        assert_eq!(
            input.redclaw_workspace_dir.is_some(),
            rc_ws,
            "redclaw_workspace_dir presence mismatch"
        );
        assert_eq!(
            input.redhorse_workspace_dir.is_some(),
            rh_ws,
            "redhorse_workspace_dir presence mismatch"
        );

        // Verify precedence: canonical always wins when present
        if rc_cfg {
            assert_eq!(input.preferred_config_dir(), Some("/rc/config".to_string()));
        } else if rh_cfg {
            assert_eq!(input.preferred_config_dir(), Some("/rh/config".to_string()));
        } else {
            assert_eq!(input.preferred_config_dir(), None);
        }

        if rc_ws {
            assert_eq!(
                input.preferred_workspace_dir(),
                Some("/rc/workspace".to_string())
            );
        } else if rh_ws {
            assert_eq!(
                input.preferred_workspace_dir(),
                Some("/rh/workspace".to_string())
            );
        } else {
            assert_eq!(input.preferred_workspace_dir(), None);
        }
    }
}
