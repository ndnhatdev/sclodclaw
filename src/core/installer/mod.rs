#![allow(unused_imports)]
#![allow(clippy::module_inception)]

//! Module installer - canonical Redhorse module lifecycle management.
//!
//! This module implements the installer workflow defined in docs/34:
//! - resolve source (local_dir, archive, bundled)
//! - obtain artifact into staging
//! - normalize artifact kind and resolved entry
//! - parse and validate manifest.json
//! - verify checksum/signature according to policy
//! - commit staged artifact into canonical module store
//! - atomically write install record into modules.lock
//! - leave disabled by default unless explicitly enabled

pub mod artifact_fetch;
pub mod installer;
pub mod quarantine;
pub mod source_resolver;
pub mod verification;

pub use artifact_fetch::*;
pub use installer::*;
pub use quarantine::*;
pub use source_resolver::*;
pub use verification::*;
