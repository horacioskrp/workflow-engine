//! Work items and their dispatch to external workers.
//!
//! Holds activatable work and streams it to workers that poll for it.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod queue;
pub mod dispatch;
