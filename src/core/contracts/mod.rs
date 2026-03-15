//! Core Redhorse contracts - type-level backbone for module system.
//!
//! This layer defines the canonical contract surface for:
//! - Module manifests, dependencies, artifacts, and execution modes
//! - Trust tiers, execution policies, and verification
//! - Orchestration contracts (sessions, turns, tool invocations)
//! - Legacy boundary normalization
//! - Adapted contracts from redclaw traits (provider, channel, tool, memory, runtime, observer)
//!
//! ## Contract Baseline Crosswalk
//!
//! Aligned with `docs/10-initial-contracts.md`:
//! - Contract sets A, B, D, F, G: defined in this layer
//! - Contract set C: split with Layer 4 (registry/lifecycle implementation)
//! - Contract set E: realized through protocol/* (Layer 6.5) + sdk/* (Layer 6)

#![allow(unused_imports)]

pub mod activation;
pub mod capability_grant_policy;
pub mod channel;
pub mod execution_mode;
pub mod execution_policy;
pub mod install_verification_policy;
pub mod memory;
pub mod message_envelope;
pub mod module_artifact;
pub mod module_capability;
pub mod module_dependency;
pub mod module_health;
pub mod module_install_record;
pub mod module_kind;
pub mod module_manifest;
pub mod module_registry;
pub mod module_trust_tier;
pub mod observer;
pub mod provider;
pub mod quarantine_state;
pub mod runtime;
pub mod runtime_services;
pub mod session_id;
pub mod session_orchestrator;
pub mod tool;
pub mod tool_invocation;
pub mod turn_context;
pub mod turn_request;
pub mod turn_result;

pub use activation::*;
pub use capability_grant_policy::*;
pub use channel::*;
pub use execution_mode::*;
pub use execution_policy::*;
pub use install_verification_policy::*;
// RedClaw-native contracts only (legacy removed in RH-MIG-T0006)
pub use memory::*;
pub use message_envelope::*;
pub use module_artifact::*;
pub use module_capability::*;
pub use module_dependency::*;
pub use module_health::*;
pub use module_install_record::*;
pub use module_kind::*;
pub use module_manifest::*;
pub use module_registry::*;
pub use module_trust_tier::*;
pub use observer::*;
pub use provider::*;
pub use quarantine_state::*;
pub use runtime::*;
pub use runtime_services::*;
pub use session_id::*;
pub use session_orchestrator::*;
pub use tool::*;
pub use tool_invocation::*;
pub use turn_context::*;
pub use turn_request::*;
pub use turn_result::*;
