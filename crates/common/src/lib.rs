//! Shared primitives: identifiers, time, and common error scaffolding.
//!
//! Foundation crate every other crate depends on; it stays free of I/O and heavy dependencies.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod ids;
pub mod time;
