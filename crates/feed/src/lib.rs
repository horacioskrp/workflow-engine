//! Outbound stream of committed history to external systems.
//!
//! Tails committed history and delivers it to downstream sinks for monitoring and analytics.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod sink;
