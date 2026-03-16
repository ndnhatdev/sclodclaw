#![warn(clippy::all, clippy::pedantic)]
#![allow(
    clippy::assigning_clones,
    clippy::bool_to_int_with_if,
    clippy::case_sensitive_file_extension_comparisons,
    clippy::cast_possible_wrap,
    clippy::doc_markdown,
    clippy::field_reassign_with_default,
    clippy::float_cmp,
    clippy::implicit_clone,
    clippy::items_after_statements,
    clippy::map_unwrap_or,
    clippy::manual_let_else,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::module_name_repetitions,
    clippy::must_use_candidate,
    clippy::new_without_default,
    clippy::needless_pass_by_value,
    clippy::needless_raw_string_hashes,
    clippy::redundant_closure_for_method_calls,
    clippy::return_self_not_must_use,
    clippy::similar_names,
    clippy::single_match_else,
    clippy::struct_field_names,
    clippy::too_many_lines,
    clippy::uninlined_format_args,
    clippy::unnecessary_cast,
    clippy::unnecessary_lazy_evaluations,
    clippy::unnecessary_literal_bound,
    clippy::unnecessary_map_or,
    clippy::unused_self,
    clippy::cast_precision_loss,
    clippy::unnecessary_wraps,
    dead_code
)]

pub mod agent;
pub(crate) mod approval;
pub(crate) mod auth;
pub mod branding;
pub mod bundled_modules;
pub mod channels;
pub mod cli_support;
mod command_surface;
pub mod config;
pub mod core;
pub(crate) mod cost;
pub(crate) mod cron;
pub(crate) mod daemon;
pub(crate) mod doctor;
pub mod gateway;
pub(crate) mod hardware;
pub(crate) mod health;
pub(crate) mod heartbeat;
pub mod hooks;
pub(crate) mod identity;
pub(crate) mod integrations;
pub mod memory;
pub(crate) mod migration;
pub(crate) mod multimodal;
pub mod observability;
pub(crate) mod onboard;
pub mod peripherals;
pub mod protocol;
pub mod providers;
pub mod rag;
pub mod runtime;
pub mod sdk;
pub(crate) mod security;
pub(crate) mod service;
pub(crate) mod skills;
pub mod tools;
pub(crate) mod tunnel;
pub(crate) mod util;

pub use command_surface::*;
pub use config::Config;
