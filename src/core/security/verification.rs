//! Module verification.

use anyhow::bail;
use std::path::Path;

pub fn verify_module(path: &Path) -> anyhow::Result<bool> {
    if !path.exists() {
        bail!("artifact path does not exist: {}", path.display());
    }

    Ok(false)
}

pub fn verify_checksum(path: &Path, expected: &str) -> anyhow::Result<bool> {
    crate::core::installer::verification::verify_checksum(path, expected)
}
