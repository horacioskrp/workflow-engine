//! History: the ordered record of everything that happened.

use serde::{Deserialize, Serialize};

/// A single committed entry in the engine's history.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct HistoryEntry {
    /// Monotonic position of this entry within its history stream.
    pub position: u64,
}
