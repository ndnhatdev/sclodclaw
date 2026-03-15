//! Public protocol boundary for app and SDK clients.
//!
//! This module defines the stable app-facing contract surface described by
//! `docs/18-public-protocol-boundary.md`.

pub mod auth;
pub mod command;
pub mod error;
pub mod event;
pub mod session;
pub mod version;

pub use auth::*;
pub use command::*;
pub use error::*;
pub use event::*;
pub use session::*;
pub use version::*;

/// GET endpoint for public health checks.
pub const PUBLIC_HEALTH_PATH: &str = "/health";
/// POST endpoint for pairing-code token exchange.
pub const PAIR_PATH: &str = "/pair";
/// GET endpoint for server-sent event streams.
pub const EVENTS_STREAM_PATH: &str = "/api/events";
/// WebSocket endpoint for interactive chat.
pub const WS_CHAT_PATH: &str = "/ws/chat";
