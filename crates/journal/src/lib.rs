//! Append-only, replicated event log stored in on-disk segments.
//!
//! The durable write-ahead log; the source of truth replicated across the cluster.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod segment;
pub mod index;
