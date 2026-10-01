//! Deterministic BPMN stream processor and element behaviors.
//!
//! The core domain: replays journal records deterministically to drive process instances.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod processor;
pub mod bpmn;
pub mod job;
pub mod process;
pub mod message;
pub mod timer;
pub mod incident;
pub mod deployment;
pub mod variable;
