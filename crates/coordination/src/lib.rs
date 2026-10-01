//! Clustering and replication of persisted state across nodes.
//!
//! Keeps replicas consistent and decides which node owns each slice of the workload.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod membership;
pub mod replication;
