//! Source resolution for module installation.

use anyhow::Result;
use std::path::{Path, PathBuf};

/// Resolve a module source string to a concrete source type.
///
/// Source formats:
/// - `@redhorse/module-name` - registry spec (future)
/// - `/absolute/path/to/module` - local directory
/// - `./relative/path` - local directory
/// - `file:///path/to/archive.tar.gz` - archive URI
/// - `/path/to/archive.tar.gz` - archive path
pub fn resolve_source(source: &str) -> Result<ResolvedSource> {
    if source.starts_with('@') {
        // Registry spec - not implemented in MVP
        anyhow::bail!(
            "registry installation not yet implemented: {}\n\
             Use local directory or archive path for now.",
            source
        );
    }

    if source.starts_with("file://") {
        // Archive URI
        let path = source.strip_prefix("file://").unwrap();
        let path = PathBuf::from(path);

        if !path.exists() {
            anyhow::bail!("archive not found: {}", path.display());
        }

        ensure_supported_archive(&path)?;

        return Ok(ResolvedSource::Archive(path));
    }

    let path = PathBuf::from(source);

    if !path.exists() {
        anyhow::bail!("source not found: {}", path.display());
    }

    if path.is_dir() {
        // Validate it has manifest.json
        let manifest_path = path.join("manifest.json");
        if !manifest_path.exists() {
            anyhow::bail!(
                "directory {} does not contain manifest.json",
                path.display()
            );
        }
        Ok(ResolvedSource::LocalDir(path))
    } else if path.is_file() {
        ensure_supported_archive(&path)?;
        Ok(ResolvedSource::Archive(path))
    } else {
        anyhow::bail!("invalid source type: {}", path.display())
    }
}

fn ensure_supported_archive(path: &Path) -> Result<()> {
    if supported_archive_format(path).is_some() {
        return Ok(());
    }

    anyhow::bail!(
        "unsupported archive format for {} (supported: .tar, .tar.gz, .tgz, .zip)",
        path.display()
    )
}

fn supported_archive_format(path: &Path) -> Option<&'static str> {
    let file_name = path.file_name()?.to_str()?.to_ascii_lowercase();

    if file_name.ends_with(".tar.gz") {
        return Some("tar.gz");
    }

    if file_name.ends_with(".tgz") {
        return Some("tgz");
    }

    let extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .map(str::to_ascii_lowercase);

    match extension.as_deref() {
        Some("tar") => Some("tar"),
        Some("zip") => Some("zip"),
        _ => None,
    }
}

/// Resolved source type.
#[derive(Debug, Clone)]
pub enum ResolvedSource {
    /// Local directory path
    LocalDir(PathBuf),
    /// Archive path
    Archive(PathBuf),
    /// Bundled module
    Bundled(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_source_accepts_supported_archive_paths() {
        let temp = tempfile::tempdir().expect("tempdir");
        let archive_path = temp.path().join("module.zip");
        std::fs::write(&archive_path, b"zip-bytes").expect("write archive");

        let resolved = resolve_source(archive_path.to_str().expect("utf8 path"))
            .expect("supported archive should resolve");
        assert!(matches!(resolved, ResolvedSource::Archive(path) if path == archive_path));
    }

    #[test]
    fn resolve_source_rejects_unsupported_archive_formats() {
        let temp = tempfile::tempdir().expect("tempdir");
        let archive_path = temp.path().join("module.rar");
        std::fs::write(&archive_path, b"rar-bytes").expect("write archive");

        let error = resolve_source(archive_path.to_str().expect("utf8 path"))
            .expect_err("unsupported archive should fail");
        assert!(
            error.to_string().contains("unsupported archive format for"),
            "unexpected error: {error:#}"
        );
    }
}
