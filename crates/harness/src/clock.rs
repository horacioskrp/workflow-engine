//! A deterministic [`Clock`](kernel::time::Clock) for reproducible tests.

use std::sync::atomic::{AtomicI64, Ordering};

use kernel::time::{Clock, Timestamp};

/// A clock whose time only moves when a test advances it.
#[derive(Debug, Default)]
pub struct ManualClock {
    millis: AtomicI64,
}

impl ManualClock {
    /// Creates a clock positioned at `millis` milliseconds since the Unix epoch.
    #[must_use]
    pub const fn starting_at(millis: i64) -> Self {
        Self {
            millis: AtomicI64::new(millis),
        }
    }

    /// Advances the clock by `millis` milliseconds.
    pub fn advance(&self, millis: i64) {
        self.millis.fetch_add(millis, Ordering::Relaxed);
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Timestamp {
        Timestamp::from_millis(self.millis.load(Ordering::Relaxed))
    }
}

#[cfg(test)]
mod tests {
    use super::ManualClock;
    use kernel::time::Clock;

    #[test]
    fn advances_only_when_told() {
        let clock = ManualClock::starting_at(0);
        assert_eq!(clock.now().as_millis(), 0);
        clock.advance(250);
        clock.advance(50);
        assert_eq!(clock.now().as_millis(), 300);
    }
}
