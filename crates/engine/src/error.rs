//! The crate's canonical error type.

use std::backtrace::Backtrace;
use std::fmt;

/// The error returned by fallible operations in this crate.
pub struct Error {
    kind: ErrorKind,
    backtrace: Backtrace,
}

#[derive(Debug)]
pub(crate) enum ErrorKind {
    /// A code path that is scaffolded but not yet implemented.
    Unimplemented,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            kind,
            backtrace: Backtrace::capture(),
        }
    }

    /// Returns an error marking a scaffolded, not-yet-implemented code path.
    #[must_use]
    pub fn unimplemented() -> Self {
        Self::new(ErrorKind::Unimplemented)
    }

    /// Returns the backtrace captured when this error was created.
    #[must_use]
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            ErrorKind::Unimplemented => f.write_str("operation not yet implemented"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error").field("kind", &self.kind).finish()
    }
}

impl std::error::Error for Error {}
