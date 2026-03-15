//! Legacy RedClaw compatibility tests
//!
//! Legacy placeholder kept outside the required docs/26 acceptance matrix.

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn test_legacy_redclaw_compat() {
        // TODO: Replace this sentinel with real compatibility coverage if the
        // legacy RedClaw surface is intentionally preserved.
        let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(
            repo_root.join("README.md").exists(),
            "legacy compatibility placeholder remains until a real matrix is added"
        );
    }
}
