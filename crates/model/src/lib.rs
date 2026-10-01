//! Domain records, intents, and the BPMN process model.
//!
//! Pure data types describing what the engine processes; serialized with serde, no behavior.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod record;
pub mod intent;
pub mod bpmn;
