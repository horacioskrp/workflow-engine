//! Records: the unit of information written to the engine's log.

use serde::{Deserialize, Serialize};

/// A single record in a partition's event log.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct Record {
    /// Monotonic position of this record within its partition log.
    pub position: u64,
}
