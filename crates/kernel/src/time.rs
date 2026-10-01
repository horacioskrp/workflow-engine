//! Time abstraction: a [`Clock`] the engine reads instead of the OS clock.
//!
//! Reading time through a trait lets tests drive it deterministically (see the
//! `harness` crate) — essential for the engine's reproducible replay.

use std::time::{SystemTime, UNIX_EPOCH};

/// A point in time, as milliseconds since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Timestamp(i64);

impl Timestamp {
    /// Builds a timestamp from milliseconds since the Unix epoch.
    #[must_use]
    pub const fn from_millis(millis: i64) -> Self {
        Self(millis)
    }

    /// Returns milliseconds since the Unix epoch.
    #[must_use]
    pub const fn as_millis(self) -> i64 {
        self.0
    }
}

/// Source of the current time.
///
/// The engine depends on this rather than the OS clock directly, so execution
/// can be replayed deterministically.
pub trait Clock: Send + Sync {
    /// Returns the current time.
    fn now(&self) -> Timestamp;
}

/// A [`Clock`] backed by the operating-system clock.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Timestamp {
        let millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |elapsed| elapsed.as_millis());
        Timestamp::from_millis(i64::try_from(millis).unwrap_or(i64::MAX))
    }
}
