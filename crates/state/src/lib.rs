//! Keyed state store with snapshot support, backed by an embedded database.
//!
//! Materialized engine state derived from the journal, plus point-in-time snapshots.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod column;
pub mod snapshot;
