//! Module registry contract surface.

use super::module_install_record::ModuleInstallRecord;

/// Registry contract for installed modules.
pub trait ModuleRegistry: Send + Sync {
    /// Returns all installed modules.
    fn installed_modules(&self) -> &[ModuleInstallRecord];

    /// Returns a module by ID.
    fn get_module(&self, id: &str) -> Option<&ModuleInstallRecord>;
}
