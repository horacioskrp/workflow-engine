//! The process model: deployable definitions clients create.

use serde::{Deserialize, Serialize};

/// A deployed process definition instances are created from.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct ProcessDefinition {
    /// Stable, human-facing identifier of the process.
    pub id: String,
}
