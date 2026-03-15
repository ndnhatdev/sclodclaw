//! Module verification for installer.

use crate::core::contracts::ModuleManifest;
use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};

/// Verify a module according to policy.
///
/// This includes:
/// - Manifest validation (already done in load_manifest)
/// - Checksum verification (future)
/// - Signature verification (future)
pub fn verify_module(manifest: &ModuleManifest, artifact_path: &Path) -> Result<()> {
    if !artifact_path.exists() {
        bail!("artifact path does not exist: {}", artifact_path.display());
    }

    // Manifest validation already happened in load_manifest
    manifest
        .validate()
        .map_err(|e| anyhow::anyhow!("manifest validation failed: {}", e))?;

    if artifact_path.is_dir() {
        let manifest_path = artifact_path.join("manifest.json");
        if !manifest_path.exists() {
            bail!(
                "artifact directory is missing manifest.json: {}",
                artifact_path.display()
            );
        }
    }

    if manifest.artifact.kind != crate::core::contracts::ArtifactKind::Bundled {
        let entry_path = artifact_path.join(&manifest.artifact.entry);
        if !entry_path.exists() {
            bail!(
                "artifact entry {} not found under {}",
                manifest.artifact.entry,
                artifact_path.display()
            );
        }
    }

    Ok(())
}

/// Verify checksum against expected value.
pub fn verify_checksum(artifact_path: &Path, expected: &str) -> Result<bool> {
    let expected = expected.trim();
    if expected.is_empty() {
        bail!("expected checksum cannot be empty");
    }

    let actual = compute_artifact_sha256(artifact_path)?;
    Ok(actual.eq_ignore_ascii_case(expected))
}

/// Verify signature.
pub fn verify_signature(artifact_path: &Path) -> Result<bool> {
    if !artifact_path.exists() {
        bail!("artifact path does not exist: {}", artifact_path.display());
    }

    let has_signature_material = signature_candidate_paths(artifact_path)
        .into_iter()
        .any(|candidate| candidate.exists());

    if has_signature_material {
        bail!(
            "signature material found for {}, but signature verification is not implemented yet",
            artifact_path.display()
        );
    }

    Ok(false)
}

pub fn compute_artifact_sha256(path: &Path) -> Result<String> {
    if path.is_file() {
        return hash_file(path);
    }

    if path.is_dir() {
        return hash_directory(path);
    }

    bail!("artifact path does not exist: {}", path.display())
}

fn hash_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("failed to open artifact file {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to read artifact file {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn hash_directory(root: &Path) -> Result<String> {
    let mut files = collect_files(root, root)?;
    files.sort_by(|left, right| left.0.cmp(&right.0));

    let mut hasher = Sha256::new();
    for (relative, path) in files {
        hasher.update(relative.as_bytes());
        hasher.update([0]);
        let bytes = fs::read(&path)
            .with_context(|| format!("failed to read artifact file {}", path.display()))?;
        hasher.update(&bytes);
        hasher.update([0xff]);
    }

    Ok(hex::encode(hasher.finalize()))
}

fn collect_files(root: &Path, current: &Path) -> Result<Vec<(String, PathBuf)>> {
    let mut files = Vec::new();
    for entry in fs::read_dir(current)
        .with_context(|| format!("failed to read artifact directory {}", current.display()))?
    {
        let entry = entry?;
        let path = entry.path();
        let relative = path
            .strip_prefix(root)
            .expect("current path should stay under root")
            .to_string_lossy()
            .replace('\\', "/");

        if entry.file_type()?.is_dir() {
            files.extend(collect_files(root, &path)?);
        } else {
            files.push((relative, path));
        }
    }
    Ok(files)
}

fn signature_candidate_paths(artifact_path: &Path) -> Vec<PathBuf> {
    if artifact_path.is_dir() {
        vec![
            artifact_path.join("manifest.sig"),
            artifact_path.join("signature.sig"),
            artifact_path.join("SIGNATURE"),
        ]
    } else {
        vec![artifact_path.with_extension("sig")]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_checksum_matches_file_contents() {
        let temp = tempfile::tempdir().expect("tempdir");
        let file_path = temp.path().join("artifact.bin");
        std::fs::write(&file_path, b"redhorse").expect("write file");

        let digest = compute_artifact_sha256(&file_path).expect("compute checksum");
        assert!(verify_checksum(&file_path, &digest).expect("verify checksum"));
    }

    #[test]
    fn verify_checksum_matches_directory_contents() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir_path = temp.path().join("artifact");
        std::fs::create_dir_all(&dir_path).expect("create dir");
        std::fs::write(dir_path.join("manifest.json"), b"{}").expect("write manifest");
        std::fs::create_dir_all(dir_path.join("bin")).expect("create nested dir");
        std::fs::write(dir_path.join("bin/module.exe"), b"payload").expect("write payload");

        let digest = compute_artifact_sha256(&dir_path).expect("compute checksum");
        assert!(verify_checksum(&dir_path, &digest).expect("verify checksum"));
    }

    #[test]
    fn verify_signature_returns_false_when_no_signature_material_exists() {
        let temp = tempfile::tempdir().expect("tempdir");
        let dir_path = temp.path().join("artifact");
        std::fs::create_dir_all(&dir_path).expect("create dir");
        std::fs::write(dir_path.join("manifest.json"), b"{}").expect("write manifest");

        assert!(!verify_signature(&dir_path).expect("verify signature"));
    }
}
