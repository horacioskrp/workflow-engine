//! Raft consensus and SWIM-style membership for partition replication.
//!
//! Turns a set of brokers into replicated partitions with a single elected leader each.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod raft;
pub mod membership;
