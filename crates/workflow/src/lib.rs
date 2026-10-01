//! Process model and deterministic execution semantics.
//!
//! The heart: advances process instances deterministically, driving tasks, timers and messages.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod activity;
pub mod execution;
pub mod incident;
pub mod model;
