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
    /// A value could not be serialized for storage.
    Encode(rmp_serde::encode::Error),
    /// A stored value could not be deserialized.
    Decode(rmp_serde::decode::Error),
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
    pub fn backtrace(&self) -> &Backtrace {
        &self.backtrace
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            ErrorKind::Unimplemented => f.write_str("operation not yet implemented"),
            ErrorKind::Encode(error) => write!(f, "failed to encode a value for storage: {error}"),
            ErrorKind::Decode(error) => write!(f, "failed to decode a stored value: {error}"),
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            ErrorKind::Unimplemented => None,
            ErrorKind::Encode(error) => Some(error),
            ErrorKind::Decode(error) => Some(error),
        }
    }
}

impl From<rmp_serde::encode::Error> for Error {
    fn from(error: rmp_serde::encode::Error) -> Self {
        Self::new(ErrorKind::Encode(error))
    }
}

impl From<rmp_serde::decode::Error> for Error {
    fn from(error: rmp_serde::decode::Error) -> Self {
        Self::new(ErrorKind::Decode(error))
    }
}
