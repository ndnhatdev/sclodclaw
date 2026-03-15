//! Channel contract - adapted from redclaw traits.

use serde::{Deserialize, Serialize};

/// A message sent through a channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChannelMessage {
    /// Sender of the message.
    pub sender: String,
    /// Message content.
    pub content: String,
}

/// Trait for sending messages through a channel.
pub trait SendMessage: Send + Sync {
    /// Sends a message.
    fn send(&self, message: ChannelMessage) -> anyhow::Result<()>;
}

/// Channel trait for communication surfaces.
pub trait Channel: SendMessage {
    /// Returns the channel name.
    fn name(&self) -> &str;

    /// Starts the channel.
    fn start(&self) -> anyhow::Result<()>;
}
