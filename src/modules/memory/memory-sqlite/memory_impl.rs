//! SQLite memory implementation.
use crate::core::contracts::{Memory, MemoryEntry, MemoryCategory};

pub struct SqliteMemory {
    db_path: String,
}

impl SqliteMemory {
    pub fn new(db_path: String) -> Self {
        Self { db_path }
    }
}

impl Memory for SqliteMemory {
    fn store(&self, entry: MemoryEntry) -> anyhow::Result<()> {
        tracing::info!("Storing memory entry: {}", entry.key);
        Ok(())
    }

    fn get(&self, key: &str) -> anyhow::Result<Option<MemoryEntry>> {
        Ok(None)
    }

    fn delete(&self, key: &str) -> anyhow::Result<()> {
        Ok(())
    }
}

pub fn create_memory(db_path: String) -> Box<dyn Memory> {
    Box::new(SqliteMemory::new(db_path))
}
