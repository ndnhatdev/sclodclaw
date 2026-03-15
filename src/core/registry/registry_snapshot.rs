use crate::core::config::modules_lock::ModulesLock;
use crate::core::contracts::ModuleInstallRecord;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrySnapshot {
    pub modules: Vec<ModuleInstallRecord>,
}

impl RegistrySnapshot {
    pub fn from_modules_lock(lock: &ModulesLock) -> Self {
        Self {
            modules: lock.modules.clone(),
        }
    }
}
