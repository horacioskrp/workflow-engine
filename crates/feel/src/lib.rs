//! FEEL expression language: parser and evaluator for BPMN expressions.
//!
//! Self-contained interpreter for the expressions embedded in process models.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod parser;
pub mod eval;
