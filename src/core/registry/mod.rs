#![allow(unused_imports)]

//! Redhorse module registry.
//!
//! This module provides the registry contract for installed modules,
//! slot ownership for exclusive categories, and catalog data.

pub mod catalog;
pub mod module_registry;
pub mod registry_snapshot;
pub mod slots;

pub use catalog::*;
pub use module_registry::*;
pub use registry_snapshot::*;
pub use slots::*;
