//! Due-time work: timers, deadlines and retry backoff.
//!
//! Tracks what must happen later and surfaces each item when its time arrives.

mod error;

pub use error::Error;

/// A specialized [`Result`] for this crate's fallible operations.
pub type Result<T, E = Error> = core::result::Result<T, E>;

pub mod retry;
pub mod timers;
