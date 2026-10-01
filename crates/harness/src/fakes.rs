//! In-memory fakes for exercising code in tests.

use std::sync::Mutex;

/// A sink that records everything pushed into it, for assertions in tests.
///
/// Useful as a stand-in wherever production code emits a stream of items
/// (committed history, dispatched work, …).
#[derive(Debug)]
pub struct RecordingSink<T> {
    items: Mutex<Vec<T>>,
}

impl<T: Clone> RecordingSink<T> {
    /// Creates an empty sink.
    #[must_use]
    pub fn new() -> Self {
        Self {
            items: Mutex::new(Vec::new()),
        }
    }

    /// Records one item.
    ///
    /// # Panics
    /// Panics if the internal mutex was poisoned by an earlier panic.
    pub fn push(&self, item: T) {
        self.items.lock().expect("sink mutex poisoned").push(item);
    }

    /// Returns a snapshot of everything recorded so far.
    ///
    /// # Panics
    /// Panics if the internal mutex was poisoned by an earlier panic.
    #[must_use]
    pub fn items(&self) -> Vec<T> {
        self.items.lock().expect("sink mutex poisoned").clone()
    }
}

impl<T: Clone> Default for RecordingSink<T> {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::RecordingSink;

    #[test]
    fn records_pushed_items_in_order() {
        let sink = RecordingSink::new();
        sink.push("first");
        sink.push("second");
        assert_eq!(sink.items(), vec!["first", "second"]);
    }
}
