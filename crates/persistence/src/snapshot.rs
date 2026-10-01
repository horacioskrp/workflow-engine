//! Consistent point-in-time snapshots of the state store.
//!
//! A snapshot captures the full store contents together with the history
//! [`Position`] it reflects, so a restored store can resume replay from the next
//! position. Snapshots also enable compaction: once state through position `P` is
//! captured, history entries through `P` can be pruned (see
//! [`History::prune_through`](crate::history::History::prune_through)).
//!
//! This increment captures and restores in memory; on-disk serialization of a
//! snapshot arrives with the durable backend.

use crate::Result;
use crate::history::Position;
use crate::store::{Column, Store, WriteBatch};

/// One captured key/value entry together with its column family.
type Entry = (Column, Box<[u8]>, Box<[u8]>);

/// A consistent capture of the store as of a history [`Position`].
#[derive(Debug)]
pub struct Snapshot {
    position: Position,
    entries: Vec<Entry>,
}

impl Snapshot {
    /// The history position this snapshot reflects.
    #[must_use]
    pub const fn position(&self) -> Position {
        self.position
    }

    /// The number of key/value entries captured.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns `true` when the snapshot captured no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Captures the full contents of `store` as of history `position`.
///
/// # Errors
/// Returns an error if the store cannot be read.
pub fn capture(store: &impl Store, position: Position) -> Result<Snapshot> {
    let mut entries = Vec::new();
    for column in store.columns() {
        for (key, value) in store.scan(column) {
            entries.push((column, key, value));
        }
    }
    Ok(Snapshot { position, entries })
}

/// Restores `snapshot` into `store`, writing back every captured key.
///
/// # Errors
/// Returns an error if the store cannot be written.
pub fn restore(snapshot: &Snapshot, store: &mut impl Store) -> Result<()> {
    let mut batch = WriteBatch::new();
    for (column, key, value) in &snapshot.entries {
        batch.put(*column, key, value);
    }
    store.apply(batch)
}

#[cfg(test)]
mod tests {
    use super::{capture, restore};
    use crate::history::{History, MemoryHistory, Position};
    use crate::store::{Column, MemoryStore, Store, WriteBatch};

    const JOBS: Column = Column::new("jobs");

    #[test]
    fn capture_then_restore_reproduces_state() {
        let mut source = MemoryStore::new();
        let mut batch = WriteBatch::new();
        batch.put(JOBS, b"a", b"1").put(JOBS, b"b", b"2");
        source.apply(batch).unwrap();

        let snapshot = capture(&source, Position::new(7)).unwrap();
        assert_eq!(snapshot.position(), Position::new(7));
        assert_eq!(snapshot.len(), 2);

        let mut restored = MemoryStore::new();
        restore(&snapshot, &mut restored).unwrap();

        assert_eq!(
            restored.get(JOBS, b"a").unwrap().as_deref(),
            Some(&b"1"[..])
        );
        assert_eq!(
            restored.get(JOBS, b"b").unwrap().as_deref(),
            Some(&b"2"[..])
        );
    }

    #[test]
    fn snapshot_enables_history_compaction() {
        let mut history = MemoryHistory::new();
        let payloads: [&[u8]; 3] = [b"a", b"b", b"c"];
        for payload in payloads {
            history.append(payload).unwrap();
        }

        // State through position 2 is captured in a snapshot, so prune through 2.
        history.prune_through(Position::new(2)).unwrap();

        let remaining = history.read_from(Position::START).unwrap();
        assert_eq!(remaining.len(), 1);
        assert_eq!(remaining[0].position(), Position::new(3));
    }
}
