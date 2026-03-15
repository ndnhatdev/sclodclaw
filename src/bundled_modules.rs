//! Compile-wired bundled module wrappers for first process-boundary slice.

#[path = "modules/providers/provider-openai-compatible/provider_impl.rs"]
pub mod provider_openai_compatible;

#[path = "modules/tools/tool-shell/tool_impl.rs"]
pub mod tool_shell;
