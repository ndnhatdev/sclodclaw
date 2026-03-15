//! CLI channel wrapper.
use redhorse_contracts::{Channel, ChannelMessage, SendMessage};

pub struct CliChannel;
impl SendMessage for CliChannel {
    fn send(&self, message: ChannelMessage) -> anyhow::Result<()> {
        println!("CLI: {} - {}", message.sender, message.content); Ok(())
    }
}
impl Channel for CliChannel {
    fn name(&self) -> &str { "cli" }
    fn start(&self) -> anyhow::Result<()> { println!("CLI channel started"); Ok(()) }
}
pub fn create_channel() -> Box<dyn Channel> { Box::new(CliChannel) }
