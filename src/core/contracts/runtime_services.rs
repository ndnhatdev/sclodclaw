//! Runtime services contract.

/// Services exposed by the runtime to modules.
pub trait RuntimeServices: Send + Sync {
    /// Whether safe mode is enabled.
    fn safe_mode_enabled(&self) -> bool;

    /// Returns the config root path.
    fn config_root(&self) -> &str;
}
