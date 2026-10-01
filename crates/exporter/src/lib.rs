//! Exporter trait and built-in record exporters.
//!
//! Streams committed records out to external systems such as a search index.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;
