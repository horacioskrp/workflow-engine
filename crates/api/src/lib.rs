//! External gRPC interface: the single entrypoint for clients.
//!
//! Accepts client calls, validates them and drives the workflow and task capabilities.

mod error;
mod service;

pub use error::Error;
pub use service::GatewayService;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;
