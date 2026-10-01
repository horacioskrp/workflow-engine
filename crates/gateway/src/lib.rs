//! gRPC gateway exposing the client API and routing to partitions.
//!
//! The single client entrypoint; translates API calls into commands on the right partition.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod service;
