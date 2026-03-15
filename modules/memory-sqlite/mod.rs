//! SQLite memory backend wrapper.
use redhorse_contracts::{Memory, MemoryEntry, MemoryCategory};

pub struct SqliteMemory { db_path: String }
impl SqliteMemory { pub fn new(db_path: String) -> Self { Self { db_path } } }
impl Memory for SqliteMemory {
    fn store(&self, _entry: MemoryEntry) -> anyhow::Result<()> { Ok(()) }
    fn get(&self, _key: &str) -> anyhow::Result<Option<MemoryEntry>> { Ok(None) }
    fn delete(&self, _key: &str) -> anyhow::Result<()> { Ok(()) }
}
pub fn create_memory(db_path: String) -> Box<dyn Memory> { Box::new(SqliteMemory::new(db_path)) }
