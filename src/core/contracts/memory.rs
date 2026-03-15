//! Memory contract - storage and retrieval.

use serde::{Deserialize, Serialize};

/// Category of memory entry.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryCategory {
    /// Core memories (permanent, high priority).
    Core,
    /// Daily memories (routine, lower priority).
    Daily,
    /// Conversation memories (chat history).
    Conversation,
    /// Custom category.
    Custom,
}

/// A single memory entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryEntry {
    /// Unique key for this memory.
    pub key: String,
    /// Memory content.
    pub content: String,
    /// Memory category.
    pub category: MemoryCategory,
}

/// Memory trait for storage and retrieval.
pub trait Memory: Send + Sync {
    /// Stores a memory entry.
    fn store(&self, entry: MemoryEntry) -> anyhow::Result<()>;

    /// Retrieves a memory entry by key.
    fn get(&self, key: &str) -> anyhow::Result<Option<MemoryEntry>>;

    /// Deletes a memory entry by key.
    fn delete(&self, key: &str) -> anyhow::Result<()>;
}
