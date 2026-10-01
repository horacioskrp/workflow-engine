//! Durable storage for engine state and process history.
//!
//! Owns how state and the history of what happened are persisted and snapshotted.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod history;
pub mod snapshot;
pub mod store;
