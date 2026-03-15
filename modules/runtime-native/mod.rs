//! Native runtime module wrapper.
use redhorse_contracts::{RuntimeAdapter, ExecutionMode};

pub struct NativeRuntime;
impl RuntimeAdapter for NativeRuntime {
    fn init(&self) -> anyhow::Result<()> { Ok(()) }
    fn run(&self, _module_id: &str) -> anyhow::Result<()> { Ok(()) }
    fn shutdown(&self) -> anyhow::Result<()> { Ok(()) }
}
pub fn create_runtime() -> Box<dyn RuntimeAdapter> { Box::new(NativeRuntime) }
