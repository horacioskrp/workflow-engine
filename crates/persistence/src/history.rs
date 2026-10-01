//! Append-only history: the ordered record of everything that happened.
//!
//! The history is the engine's source of truth; materialized state is derived
//! from it by replaying entries in order. This module defines the [`History`]
//! seam and an in-memory implementation; a durable backend lands later in
//! Phase 1.

use serde::{Deserialize, Serialize};

use crate::Result;

/// A position in the history, assigned on append. Monotonic, starting at 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Position(u64);

impl Position {
    /// The position before the first entry; `read_from(Position::START)` yields
    /// the whole history.
    pub const START: Self = Self(0);

    /// Wraps a raw position.
    #[must_use]
    pub const fn new(raw: u64) -> Self {
        Self(raw)
    }

    /// Returns the raw position.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A committed history entry: its position plus an opaque payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HistoryEntry {
    position: Position,
    payload: Box<[u8]>,
}

impl HistoryEntry {
    /// The position this entry was committed at.
    #[must_use]
    pub const fn position(&self) -> Position {
        self.position
    }

    /// The entry's opaque payload.
    #[must_use]
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }
}

/// An append-only, ordered log of entries — the engine's source of truth.
pub trait History {
    /// Appends `payload` and returns the position it was committed at.
    ///
    /// # Errors
    /// Returns an error if the underlying store cannot persist the entry.
    fn append(&mut self, payload: &[u8]) -> Result<Position>;

    /// Returns every entry committed at a position `>= from`, in order.
    ///
    /// # Errors
    /// Returns an error if the underlying store cannot be read.
    fn read_from(&self, from: Position) -> Result<Vec<HistoryEntry>>;

    /// Returns the last committed position, or [`Position::START`] when empty.
    fn last_position(&self) -> Position;

    /// Drops every entry at a position `<= through`, keeping later ones.
    ///
    /// Used for compaction once state through `through` is captured in a snapshot.
    ///
    /// # Errors
    /// Returns an error if the underlying store cannot be pruned.
    fn prune_through(&mut self, through: Position) -> Result<()>;
}

/// An in-memory [`History`] for tests and early development.
#[derive(Debug, Default)]
pub struct MemoryHistory {
    entries: Vec<HistoryEntry>,
    last: u64,
}

impl MemoryHistory {
    /// Creates an empty history.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl History for MemoryHistory {
    fn append(&mut self, payload: &[u8]) -> Result<Position> {
        self.last += 1;
        let position = Position(self.last);
        self.entries.push(HistoryEntry {
            position,
            payload: payload.into(),
        });
        Ok(position)
    }

    fn read_from(&self, from: Position) -> Result<Vec<HistoryEntry>> {
        Ok(self
            .entries
            .iter()
            .filter(|entry| entry.position >= from)
            .cloned()
            .collect())
    }

    fn last_position(&self) -> Position {
        Position(self.last)
    }

    fn prune_through(&mut self, through: Position) -> Result<()> {
        self.entries.retain(|entry| entry.position > through);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{History, MemoryHistory, Position};

    #[test]
    fn append_assigns_monotonic_positions() {
        let mut history = MemoryHistory::new();
        assert_eq!(history.append(b"a").unwrap(), Position::new(1));
        assert_eq!(history.append(b"b").unwrap(), Position::new(2));
        assert_eq!(history.last_position(), Position::new(2));
    }

    #[test]
    fn replay_from_start_returns_entries_in_order() {
        let mut history = MemoryHistory::new();
        let payloads: [&[u8]; 3] = [b"one", b"two", b"three"];
        for payload in payloads {
            history.append(payload).unwrap();
        }

        let replayed: Vec<Vec<u8>> = history
            .read_from(Position::START)
            .unwrap()
            .into_iter()
            .map(|entry| entry.payload().to_vec())
            .collect();

        assert_eq!(
            replayed,
            vec![b"one".to_vec(), b"two".to_vec(), b"three".to_vec()]
        );
    }

    #[test]
    fn read_from_skips_earlier_positions() {
        let mut history = MemoryHistory::new();
        history.append(b"a").unwrap();
        let second = history.append(b"b").unwrap();

        let tail = history.read_from(second).unwrap();

        assert_eq!(tail.len(), 1);
        assert_eq!(tail[0].payload(), b"b");
    }
}
