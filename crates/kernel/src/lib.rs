//! Shared primitives: identifiers, clocks and error scaffolding.
//!
//! Foundation crate every other crate builds on; free of I/O and heavy
//! dependencies. (Telemetry is the `tracing` facade in libraries, with the
//! subscriber installed by the binaries.)

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod ids;
pub mod time;
