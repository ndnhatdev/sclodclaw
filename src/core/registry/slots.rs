//! Slot ownership for exclusive module categories.

use std::collections::HashMap;

/// A slot for exclusive module categories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Slot {
    /// Slot name (e.g., "runtime", "default-provider").
    pub name: String,
    /// Module ID occupying this slot.
    pub module_id: String,
    /// Whether the slot is active.
    pub active: bool,
}

/// Slot manager for exclusive categories.
pub struct SlotManager {
    slots: HashMap<String, Slot>,
}

impl SlotManager {
    /// Creates a new slot manager.
    pub fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }

    /// Claims a slot for a module.
    pub fn claim(&mut self, slot_name: &str, module_id: &str) -> Result<(), String> {
        if let Some(existing) = self.slots.get(slot_name) {
            if existing.module_id != module_id {
                return Err(format!(
                    "Slot '{}' is already occupied by module '{}'",
                    slot_name, existing.module_id
                ));
            }
        }

        self.slots.insert(
            slot_name.to_string(),
            Slot {
                name: slot_name.to_string(),
                module_id: module_id.to_string(),
                active: true,
            },
        );
        Ok(())
    }

    /// Releases a slot.
    pub fn release(&mut self, slot_name: &str) {
        self.slots.remove(slot_name);
    }

    /// Returns the module occupying a slot.
    pub fn get_occupant(&self, slot_name: &str) -> Option<&str> {
        self.slots.get(slot_name).map(|s| s.module_id.as_str())
    }
}

impl Default for SlotManager {
    fn default() -> Self {
        Self::new()
    }
}
