//! CLI channel implementation.
use crate::core::contracts::{Channel, ChannelMessage, SendMessage};

pub struct CliChannel;

impl SendMessage for CliChannel {
    fn send(&self, message: ChannelMessage) -> anyhow::Result<()> {
        println!("CLI Channel: {} - {}", message.sender, message.content);
        Ok(())
    }
}

impl Channel for CliChannel {
    fn name(&self) -> &str {
        "cli"
    }

    fn start(&self) -> anyhow::Result<()> {
        tracing::info!("CLI channel started");
        Ok(())
    }
}

pub fn create_channel() -> Box<dyn Channel> {
    Box::new(CliChannel)
}
