//! Signals and messages with correlation to waiting instances.
//!
//! Buffers published messages and correlates them to the instances awaiting them.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod subscription;
pub mod correlation;
