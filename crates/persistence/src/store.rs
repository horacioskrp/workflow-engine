//! Keyed state store: the materialized engine state, derived from the history.
//!
//! State is organized into named [`Column`] families (like `RocksDB` column
//! families). Writes are grouped into a [`WriteBatch`] and applied atomically,
//! so a record's effects land all-or-nothing. Reads are exposed as **ordered
//! iterators** ([`Store::scan`], [`Store::scan_prefix`]); ordering is a
//! `BTreeMap`, so iteration is deterministic — a prerequisite for the engine's
//! reproducible replay. [`TypedColumn`] adds a serde-typed view over a family so
//! callers work with real key/value types instead of raw bytes.
//!
//! This module defines the [`Store`] seam and an in-memory implementation; a
//! durable backend lands later in Phase 1.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use serde::Serialize;
use serde::de::DeserializeOwned;

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

/// An owned key/value pair, as yielded by [`Store::scan`].
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

    /// Iterates every key/value in `column`, ordered by key.
    fn scan(&self, column: Column) -> impl Iterator<Item = KeyValue> + '_;

    /// Iterates entries in `column` whose key starts with `prefix`, ordered by key.
    fn scan_prefix<'s>(
        &'s self,
        column: Column,
        prefix: &[u8],
    ) -> impl Iterator<Item = KeyValue> + 's;
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

    fn scan(&self, column: Column) -> impl Iterator<Item = KeyValue> + '_ {
        self.families
            .get(&column)
            .into_iter()
            .flat_map(|family| family.iter())
            .map(|(key, value)| (key.clone(), value.clone()))
    }

    fn scan_prefix<'s>(
        &'s self,
        column: Column,
        prefix: &[u8],
    ) -> impl Iterator<Item = KeyValue> + 's {
        let prefix = prefix.to_vec();
        self.families
            .get(&column)
            .into_iter()
            .flat_map(|family| family.iter())
            .filter(move |(key, _)| key.starts_with(prefix.as_slice()))
            .map(|(key, value)| (key.clone(), value.clone()))
    }
}

/// A serde-typed view over a [`Column`]: keys and values are encoded with
/// `MessagePack`, so callers work with real types instead of raw bytes.
pub struct TypedColumn<K, V> {
    column: Column,
    _marker: PhantomData<fn() -> (K, V)>,
}

impl<K, V> TypedColumn<K, V> {
    /// Creates a typed view over `column`.
    #[must_use]
    pub const fn new(column: Column) -> Self {
        Self {
            column,
            _marker: PhantomData,
        }
    }
}

impl<K, V> TypedColumn<K, V>
where
    K: Serialize + DeserializeOwned,
    V: Serialize + DeserializeOwned,
{
    /// Returns the value for `key`, if present.
    ///
    /// # Errors
    /// Returns an error if the key or stored value cannot be (de)serialized, or
    /// if the store cannot be read.
    pub fn get(&self, store: &impl Store, key: &K) -> Result<Option<V>> {
        let raw_key = rmp_serde::to_vec(key)?;
        match store.get(self.column, &raw_key)? {
            Some(raw_value) => Ok(Some(rmp_serde::from_slice(&raw_value)?)),
            None => Ok(None),
        }
    }

    /// Stages a typed put into `batch`.
    ///
    /// # Errors
    /// Returns an error if the key or value cannot be serialized.
    pub fn put(&self, batch: &mut WriteBatch, key: &K, value: &V) -> Result<()> {
        let raw_key = rmp_serde::to_vec(key)?;
        let raw_value = rmp_serde::to_vec(value)?;
        batch.put(self.column, &raw_key, &raw_value);
        Ok(())
    }

    /// Stages a typed delete into `batch`.
    ///
    /// # Errors
    /// Returns an error if the key cannot be serialized.
    pub fn delete(&self, batch: &mut WriteBatch, key: &K) -> Result<()> {
        let raw_key = rmp_serde::to_vec(key)?;
        batch.delete(self.column, &raw_key);
        Ok(())
    }

    /// Iterates the family's decoded key/value pairs, ordered by encoded key.
    ///
    /// Each item is a [`Result`] because a stored value may fail to decode.
    pub fn iter<'a, S>(&self, store: &'a S) -> impl Iterator<Item = Result<(K, V)>> + 'a
    where
        S: Store,
    {
        store
            .scan(self.column)
            .map(|(raw_key, raw_value)| -> Result<(K, V)> {
                let key: K = rmp_serde::from_slice(&raw_key)?;
                let value: V = rmp_serde::from_slice(&raw_value)?;
                Ok((key, value))
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{Column, MemoryStore, Store, TypedColumn, WriteBatch};

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

        let keys: Vec<Vec<u8>> = store.scan(JOBS).map(|(key, _)| key.to_vec()).collect();

        assert_eq!(keys, vec![b"a".to_vec(), b"b".to_vec(), b"c".to_vec()]);
    }

    #[test]
    fn scan_prefix_filters_by_key_prefix() {
        let mut store = MemoryStore::new();
        let mut batch = WriteBatch::new();
        batch
            .put(JOBS, b"job:1", b"x")
            .put(JOBS, b"job:2", b"y")
            .put(JOBS, b"timer:1", b"z");
        store.apply(batch).unwrap();

        let keys: Vec<Vec<u8>> = store
            .scan_prefix(JOBS, b"job:")
            .map(|(key, _)| key.to_vec())
            .collect();

        assert_eq!(keys, vec![b"job:1".to_vec(), b"job:2".to_vec()]);
    }

    #[test]
    fn typed_column_roundtrips_and_iterates() {
        let timers: TypedColumn<u64, String> = TypedColumn::new(Column::new("timers"));
        let mut store = MemoryStore::new();

        let mut batch = WriteBatch::new();
        timers.put(&mut batch, &2, &"two".to_owned()).unwrap();
        timers.put(&mut batch, &1, &"one".to_owned()).unwrap();
        store.apply(batch).unwrap();

        assert_eq!(timers.get(&store, &1).unwrap(), Some("one".to_owned()));
        assert_eq!(timers.get(&store, &99).unwrap(), None);

        let items: Vec<(u64, String)> = timers.iter(&store).map(Result::unwrap).collect();
        assert_eq!(items.len(), 2);
        assert!(items.contains(&(1, "one".to_owned())));
        assert!(items.contains(&(2, "two".to_owned())));
    }
}
