//! Quarantine management for failed verification.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Move artifact to quarantine.
pub fn quarantine_artifact(source: &Path, quarantine_dir: &Path, reason: &str) -> Result<PathBuf> {
    fs::create_dir_all(quarantine_dir).context("failed to create quarantine directory")?;

    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let quarantine_name = format!(
        "{}_{}",
        timestamp,
        source.file_name().unwrap().to_string_lossy()
    );
    let quarantine_path = quarantine_dir.join(&quarantine_name);

    if fs::rename(source, &quarantine_path).is_err() {
        // Fallback to copy if rename fails
        copy_dir_all(source, &quarantine_path)?;
        fs::remove_dir_all(source)?;
    }

    // Write reason file
    let reason_path = quarantine_path.join("QUARANTINE_REASON.txt");
    fs::write(
        reason_path,
        format!("Quarantined: {}\nReason: {}\n", timestamp, reason),
    )?;

    Ok(quarantine_path)
}

/// List quarantined artifacts.
pub fn list_quarantine(quarantine_dir: &Path) -> Result<Vec<QuarantinedArtifact>> {
    let mut artifacts = Vec::new();

    if !quarantine_dir.exists() {
        return Ok(artifacts);
    }

    for entry in fs::read_dir(quarantine_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let reason = read_quarantine_reason(&path);
            artifacts.push(QuarantinedArtifact { path, reason });
        }
    }

    Ok(artifacts)
}

/// Read quarantine reason from artifact directory.
fn read_quarantine_reason(quarantine_path: &Path) -> Option<String> {
    let reason_path = quarantine_path.join("QUARANTINE_REASON.txt");
    fs::read_to_string(reason_path).ok()
}

/// Quarantined artifact info.
#[derive(Debug, Clone)]
pub struct QuarantinedArtifact {
    pub path: PathBuf,
    pub reason: Option<String>,
}

/// Recursively copy a directory.
fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;

        if ty.is_dir() {
            copy_dir_all(&entry.path(), &dst.join(entry.file_name()))?;
        } else {
            fs::copy(entry.path(), dst.join(entry.file_name()))?;
        }
    }

    Ok(())
}
