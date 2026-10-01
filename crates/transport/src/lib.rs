//! Node-to-node network transport for the broker cluster.
//!
//! Framed messaging between brokers; the substrate the cluster layer builds on.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;
