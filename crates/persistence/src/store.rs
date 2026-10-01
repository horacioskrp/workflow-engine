//! Keyed state store: the materialized engine state, derived from the history.
//!
//! State is organized into named [`Column`] families (like `RocksDB` column
//! families). Writes are grouped into a [`WriteBatch`] and applied atomically,
//! so a record's effects land all-or-nothing. This module defines the [`Store`]
//! seam and an ordered in-memory implementation; a durable backend lands later
//! in Phase 1. Ordering is a `BTreeMap`, so scans are deterministic — a
//! prerequisite for the engine's reproducible replay.

use std::collections::BTreeMap;

use crate::Result;

/// Identifies a column family — a named namespace of keys within the store.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Column(&'static str);

impl Column {
    /// Names a column family.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Returns the family name.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.0
    }
}

/// An owned key/value pair, as returned by [`Store::scan`].
pub type KeyValue = (Box<[u8]>, Box<[u8]>);

/// One column family's ordered keys (internal storage representation).
type Family = BTreeMap<Box<[u8]>, Box<[u8]>>;

#[derive(Debug)]
enum Op {
    Put {
        column: Column,
        key: Box<[u8]>,
        value: Box<[u8]>,
    },
    Delete {
        column: Column,
        key: Box<[u8]>,
    },
}

/// A set of writes applied to a [`Store`] atomically.
#[derive(Debug, Default)]
pub struct WriteBatch {
    ops: Vec<Op>,
}

impl WriteBatch {
    /// Creates an empty batch.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Stages a put of `value` at `key` in `column`.
    pub fn put(&mut self, column: Column, key: &[u8], value: &[u8]) -> &mut Self {
        self.ops.push(Op::Put {
            column,
            key: key.into(),
            value: value.into(),
        });
        self
    }

    /// Stages a delete of `key` in `column`.
    pub fn delete(&mut self, column: Column, key: &[u8]) -> &mut Self {
        self.ops.push(Op::Delete {
            column,
            key: key.into(),
        });
        self
    }

    /// Returns the number of staged operations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ops.len()
    }

    /// Returns `true` when no operation is staged.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }
}

/// A keyed, column-organized state store with atomic batch writes.
pub trait Store {
    /// Returns the value at `key` in `column`, if present.
    ///
    /// # Errors
    /// Returns an error if the underlying store cannot be read.
    fn get(&self, column: Column, key: &[u8]) -> Result<Option<Box<[u8]>>>;

    /// Applies every operation in `batch` atomically.
    ///
    /// # Errors
    /// Returns an error if the underlying store cannot persist the batch.
    fn apply(&mut self, batch: WriteBatch) -> Result<()>;

    /// Returns every key/value in `column`, ordered by key.
    ///
    /// # Errors
    /// Returns an error if the underlying store cannot be read.
    fn scan(&self, column: Column) -> Result<Vec<KeyValue>>;
}

/// An ordered, in-memory [`Store`] for tests and early development.
#[derive(Debug, Default)]
pub struct MemoryStore {
    families: BTreeMap<Column, Family>,
}

impl MemoryStore {
    /// Creates an empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl Store for MemoryStore {
    fn get(&self, column: Column, key: &[u8]) -> Result<Option<Box<[u8]>>> {
        Ok(self
            .families
            .get(&column)
            .and_then(|family| family.get(key))
            .cloned())
    }

    fn apply(&mut self, batch: WriteBatch) -> Result<()> {
        for op in batch.ops {
            match op {
                Op::Put { column, key, value } => {
                    self.families.entry(column).or_default().insert(key, value);
                }
                Op::Delete { column, key } => {
                    if let Some(family) = self.families.get_mut(&column) {
                        family.remove(&key);
                    }
                }
            }
        }
        Ok(())
    }

    fn scan(&self, column: Column) -> Result<Vec<KeyValue>> {
        Ok(self.families.get(&column).map_or_else(Vec::new, |family| {
            family
                .iter()
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, MemoryStore, Store, WriteBatch};

    const JOBS: Column = Column::new("jobs");

    #[test]
    fn put_then_get_roundtrips() {
        let mut store = MemoryStore::new();
        let mut batch = WriteBatch::new();
        batch.put(JOBS, b"k1", b"v1");
        store.apply(batch).unwrap();

        assert_eq!(store.get(JOBS, b"k1").unwrap().as_deref(), Some(&b"v1"[..]));
        assert_eq!(store.get(JOBS, b"missing").unwrap(), None);
    }

    #[test]
    fn apply_is_atomic_batch_of_puts_and_deletes() {
        let mut store = MemoryStore::new();
        let mut seed = WriteBatch::new();
        seed.put(JOBS, b"a", b"1").put(JOBS, b"b", b"2");
        store.apply(seed).unwrap();

        let mut batch = WriteBatch::new();
        batch.put(JOBS, b"c", b"3").delete(JOBS, b"a");
        assert_eq!(batch.len(), 2);
        store.apply(batch).unwrap();

        assert_eq!(store.get(JOBS, b"a").unwrap(), None);
        assert_eq!(store.get(JOBS, b"c").unwrap().as_deref(), Some(&b"3"[..]));
    }

    #[test]
    fn scan_returns_entries_ordered_by_key() {
        let mut store = MemoryStore::new();
        let mut batch = WriteBatch::new();
        batch
            .put(JOBS, b"c", b"3")
            .put(JOBS, b"a", b"1")
            .put(JOBS, b"b", b"2");
        store.apply(batch).unwrap();

        let keys: Vec<Vec<u8>> = store
            .scan(JOBS)
            .unwrap()
            .into_iter()
            .map(|(key, _)| key.to_vec())
            .collect();

        assert_eq!(keys, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);
    }
}
