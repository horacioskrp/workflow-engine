//! Rust client SDK for the gateway API.
//!
//! Ergonomic async client used by workers and applications to drive the engine.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;
