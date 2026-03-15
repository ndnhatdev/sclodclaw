//! RedClaw-native environment variable reader.
//!
//! # RedClaw-Only Policy
//!
//! This module implements **RedClaw-exclusive** environment variable resolution.
//! NO legacy compatibility (REDHORSE_*, ZEROCLAW_*) is provided.
//!
//! ## Supported Variables
//!
//! - `REDCLAW_CONFIG_DIR` - Override config directory
//! - `REDCLAW_WORKSPACE` - Override workspace directory
//!
//! ## Precedence
//!
//! 1. `REDCLAW_CONFIG_DIR` env var (highest priority)
//! 2. `REDCLAW_WORKSPACE` env var
//! 3. `active_workspace.toml` marker (persisted user choice)
//! 4. `~/.redclaw` default (canonical home directory)
//!
//! # No Legacy Fallback
//!
//! Legacy environment variables (REDHORSE_*, ZEROCLAW_*) are **NOT SUPPORTED**.
//! Users must migrate to REDCLAW_* variables.
//!
//! # Policy
//!
//! - RedClaw-native only (RH-MIG-T0009 Option A: zero-legacy in new code)
//! - NO deprecation warnings (legacy vars are ignored, not warned)
//! - Fail closed on malformed inputs (return None, do not crash)
//!
//! # Handoff
//!
//! This is the permanent config resolution layer. Legacy compatibility
//! was removed in RH-MIG-T0006 after migration matrix showed green.

use serde::{Deserialize, Serialize};

/// RedClaw environment variable input collector.
///
/// Captures only canonical REDCLAW_* env vars.
/// Values are trimmed and validated (empty strings become None).
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EnvInput {
    /// Canonical: REDCLAW_CONFIG_DIR
    pub config_dir: Option<String>,
    /// Canonical: REDCLAW_WORKSPACE
    pub workspace_dir: Option<String>,
}

impl EnvInput {
    /// Collect all RedClaw environment variables.
    ///
    /// # Returns
    ///
    /// Returns structured data for canonical env vars only.
    /// Legacy vars (REDHORSE_*, ZEROCLAW_*) are IGNORED.
    pub fn collect() -> Self {
        let config_dir = read_trimmed("REDCLAW_CONFIG_DIR");
        let workspace_dir = read_trimmed("REDCLAW_WORKSPACE");

        Self {
            config_dir,
            workspace_dir,
        }
    }

    /// Returns the preferred config dir.
    ///
    /// # Precedence
    ///
    /// 1. REDCLAW_CONFIG_DIR (canonical)
    ///
    /// Returns None if not set.
    pub fn config_dir(&self) -> Option<String> {
        self.config_dir.clone()
    }

    /// Returns the preferred workspace dir.
    ///
    /// # Precedence
    ///
    /// 1. REDCLAW_WORKSPACE (canonical)
    ///
    /// Returns None if not set.
    pub fn workspace_dir(&self) -> Option<String> {
        self.workspace_dir.clone()
    }
}

/// Read and trim a RedClaw environment variable.
///
/// Returns None if unset, empty after trim, or malformed.
fn read_trimmed(name: &str) -> Option<String> {
    std::env::var(name)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

/// Returns a human-readable note about env var usage.
///
/// Used in CLI help text, diagnostics, or migration docs.
pub fn canonical_env_note() -> String {
    "RedClaw uses REDCLAW_* environment variables. See docs for configuration.".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    fn cleanup_env(vars: &[&str]) {
        for var in vars {
            env::remove_var(var);
        }
    }

    #[test]
    fn test_collect_no_env_vars() {
        let vars = ["REDCLAW_CONFIG_DIR", "REDCLAW_WORKSPACE"];
        cleanup_env(&vars);

        let input = EnvInput::collect();
        assert_eq!(input, EnvInput::default());
    }

    #[test]
    fn test_collect_canonical_vars() {
        let vars = ["REDCLAW_CONFIG_DIR", "REDCLAW_WORKSPACE"];
        cleanup_env(&vars);

        env::set_var("REDCLAW_CONFIG_DIR", "/canonical/config");
        env::set_var("REDCLAW_WORKSPACE", "/canonical/workspace");

        let input = EnvInput::collect();
        assert_eq!(input.config_dir, Some("/canonical/config".to_string()));
        assert_eq!(
            input.workspace_dir,
            Some("/canonical/workspace".to_string())
        );

        cleanup_env(&vars);
    }

    #[test]
    fn test_trim_and_empty_filter() {
        let vars = ["REDCLAW_CONFIG_DIR"];
        cleanup_env(&vars);

        // Whitespace-only becomes None
        env::set_var("REDCLAW_CONFIG_DIR", "   ");
        let input = EnvInput::collect();
        assert_eq!(input.config_dir, None);
        cleanup_env(&vars);
    }

    #[test]
    fn test_legacy_vars_ignored() {
        let vars = [
            "REDCLAW_CONFIG_DIR",
            "REDHORSE_CONFIG_DIR",
            "ZEROCLAW_CONFIG_DIR",
        ];
        cleanup_env(&vars);

        // Set ONLY legacy vars - should be IGNORED
        env::set_var("REDHORSE_CONFIG_DIR", "/legacy/config");
        env::set_var("ZEROCLAW_CONFIG_DIR", "/legacy/zeroclaw");

        let input = EnvInput::collect();
        assert_eq!(input.config_dir, None); // Legacy vars ignored
        assert_eq!(input.workspace_dir, None);

        cleanup_env(&vars);
    }
}
