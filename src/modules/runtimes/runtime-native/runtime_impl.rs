//! Native runtime implementation.
use crate::core::contracts::{RuntimeAdapter, ExecutionMode};

pub struct NativeRuntime;

impl RuntimeAdapter for NativeRuntime {
    fn init(&self) -> anyhow::Result<()> {
        tracing::info!("Native runtime initialized");
        Ok(())
    }

    fn run(&self, module_id: &str) -> anyhow::Result<()> {
        tracing::info!("Running module: {}", module_id);
        Ok(())
    }

    fn shutdown(&self) -> anyhow::Result<()> {
        tracing::info!("Native runtime shutdown");
        Ok(())
    }
}

pub fn create_runtime() -> Box<dyn RuntimeAdapter> {
    Box::new(NativeRuntime)
}
