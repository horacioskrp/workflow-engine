//! Shared primitives: identifiers, clocks, telemetry and error scaffolding.
//!
//! Foundation crate every other crate builds on; free of I/O and heavy dependencies.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod ids;
pub mod telemetry;
pub mod time;
