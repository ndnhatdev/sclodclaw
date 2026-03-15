//! Module manifest validation.

use crate::core::contracts::ModuleManifest;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}

pub fn validate_manifest(manifest: &ModuleManifest) -> ValidationResult {
    match manifest.validate() {
        Ok(()) => ValidationResult {
            valid: true,
            errors: vec![],
        },
        Err(err) => ValidationResult {
            valid: false,
            errors: vec![err],
        },
    }
}
