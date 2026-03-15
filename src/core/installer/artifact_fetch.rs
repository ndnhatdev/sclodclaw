//! Artifact fetch helpers for module installation.

use super::source_resolver::ResolvedSource;
use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Fetch artifact from resolved source to staging directory.
///
/// Returns the path to the staged artifact.
pub fn fetch_artifact(source: &ResolvedSource, staging_dir: &Path) -> Result<PathBuf> {
    match source {
        ResolvedSource::LocalDir(path) => fetch_local_dir(path, staging_dir),
        ResolvedSource::Archive(path) => fetch_archive(path, staging_dir),
        ResolvedSource::Bundled(_) => {
            anyhow::bail!("bundled module fetch not implemented")
        }
    }
}

/// Copy local directory to staging.
fn fetch_local_dir(source: &Path, staging_dir: &Path) -> Result<PathBuf> {
    let dest = staging_dir.join("staged-module");

    // Clean staging
    if dest.exists() {
        fs::remove_dir_all(&dest).ok();
    }

    // Copy directory
    copy_dir_all(source, &dest).context("failed to copy module to staging")?;

    Ok(dest)
}

/// Extract archive to staging.
fn fetch_archive(archive_path: &Path, staging_dir: &Path) -> Result<PathBuf> {
    let dest = staging_dir.join("staged-module");

    // Clean staging
    if dest.exists() {
        fs::remove_dir_all(&dest).ok();
    }

    fs::create_dir_all(&dest).context("failed to create archive staging directory")?;

    // Determine archive type and extract
    let extension = archive_extension(archive_path);

    match extension.as_str() {
        "tar" | "tar.gz" | "tgz" => extract_tar(archive_path, &dest)?,
        "zip" => extract_zip(archive_path, &dest)?,
        _ => anyhow::bail!(
            "unsupported archive format for {} (supported: .tar, .tar.gz, .tgz, .zip)",
            archive_path.display()
        ),
    }

    flatten_single_root(&dest)?;

    Ok(dest)
}

/// Extract tar/tar.gz archive.
fn extract_tar(archive_path: &Path, staging_dir: &Path) -> Result<()> {
    use std::process::Command;

    let flag = if matches!(archive_extension(archive_path).as_str(), "tar.gz" | "tgz") {
        "-xzf"
    } else {
        "-xf"
    };

    let status = Command::new("tar")
        .arg(flag)
        .arg(archive_path)
        .arg("-C")
        .arg(staging_dir)
        .status()
        .with_context(|| format!("failed to spawn tar for {}", archive_path.display()))?;

    if !status.success() {
        anyhow::bail!("tar extraction failed for {}", archive_path.display());
    }

    Ok(())
}

/// Extract zip archive.
fn extract_zip(archive_path: &Path, staging_dir: &Path) -> Result<()> {
    // Simple zip extraction using std::process with unzip command
    // For production, would use zip crate
    #[cfg(unix)]
    {
        use std::process::Command;
        let status = Command::new("unzip")
            .arg(archive_path)
            .arg("-d")
            .arg(staging_dir)
            .status()
            .with_context(|| format!("failed to spawn unzip for {}", archive_path.display()))?;

        if !status.success() {
            anyhow::bail!("zip extraction failed for {}", archive_path.display());
        }
        Ok(())
    }

    #[cfg(windows)]
    {
        // Windows: use PowerShell Expand-Archive
        use std::process::Command;
        let status = Command::new("powershell")
            .arg("-Command")
            .arg("Expand-Archive")
            .arg("-Path")
            .arg(archive_path)
            .arg("-DestinationPath")
            .arg(staging_dir)
            .arg("-Force")
            .status()
            .with_context(|| {
                format!(
                    "failed to spawn PowerShell Expand-Archive for {}",
                    archive_path.display()
                )
            })?;

        if !status.success() {
            anyhow::bail!("zip extraction failed for {}", archive_path.display());
        }
        Ok(())
    }
}

fn archive_extension(path: &Path) -> String {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .map(str::to_ascii_lowercase)
        .unwrap_or_default();

    if file_name.ends_with(".tar.gz") {
        "tar.gz".to_string()
    } else if file_name.ends_with(".tgz") {
        "tgz".to_string()
    } else {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(str::to_ascii_lowercase)
            .unwrap_or_default()
    }
}

fn flatten_single_root(root: &Path) -> Result<()> {
    let mut entries = fs::read_dir(root)?
        .collect::<std::io::Result<Vec<_>>>()?
        .into_iter()
        .collect::<Vec<_>>();

    if entries.len() != 1 || !entries[0].file_type()?.is_dir() {
        return Ok(());
    }

    let nested_root = entries.pop().expect("single entry checked").path();
    for entry in fs::read_dir(&nested_root)? {
        let entry = entry?;
        fs::rename(entry.path(), root.join(entry.file_name()))?;
    }
    fs::remove_dir_all(nested_root)?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn archive_extension_recognizes_tar_gz_and_tgz() {
        assert_eq!(archive_extension(Path::new("module.tar.gz")), "tar.gz");
        assert_eq!(archive_extension(Path::new("module.tgz")), "tgz");
        assert_eq!(archive_extension(Path::new("module.zip")), "zip");
        assert_eq!(archive_extension(Path::new("module.tar")), "tar");
    }

    #[test]
    fn flatten_single_root_moves_nested_contents_up() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        let nested = root.join("nested");
        std::fs::create_dir_all(&nested).expect("create nested root");
        std::fs::write(nested.join("manifest.json"), "{}").expect("write manifest");

        flatten_single_root(root).expect("flatten root");

        assert!(root.join("manifest.json").exists());
        assert!(!nested.exists());
    }

    #[test]
    fn flatten_single_root_keeps_multiple_entries_intact() {
        let temp = tempfile::tempdir().expect("tempdir");
        let root = temp.path();
        std::fs::write(root.join("manifest.json"), "{}").expect("write manifest");
        std::fs::write(root.join("extra.txt"), "hello").expect("write extra");

        flatten_single_root(root).expect("flatten root");

        assert!(root.join("manifest.json").exists());
        assert!(root.join("extra.txt").exists());
    }
}
