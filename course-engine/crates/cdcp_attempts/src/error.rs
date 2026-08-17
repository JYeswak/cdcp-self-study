//! Named failures. An empty store is an ERROR, never a zero-N analysis.

use thiserror::Error;

/// Token interpolated in the empty-store path. Deleting the check makes the
/// matching known-bad test non-zero.
pub const EMPTY_STORE: &str = "empty attempt store is a schema ERROR";

/// Token interpolated when export is requested without opt-in.
pub const EXPORT_NOT_OPTED_IN: &str = "export requires explicit opt-in (default OFF)";

/// Token interpolated when a psychometric computation is requested.
pub const PSYCHOMETRICS_REFUSED: &str =
    "this crate captures events; it refuses IRT, item difficulty, and discrimination";

/// Why an event, store, or export could not be produced.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AttemptError {
    /// A required string field was empty or whitespace-only.
    #[error("empty field is a schema ERROR: {0}")]
    EmptyField(&'static str),
    /// `timestamp_unix_ms == 0` means the clock was never written.
    #[error("timestamp_unix_ms == 0 is a schema ERROR (clock unset)")]
    TimestampUnset,
    /// Read or export of a store with zero events.
    #[error("{EMPTY_STORE}")]
    EmptyStore,
    /// Export was requested while [`crate::ExportPolicy`] is OFF.
    #[error("{EXPORT_NOT_OPTED_IN}")]
    ExportNotOptedIn,
    /// IRT / difficulty / discrimination — always refused, at any N.
    #[error("{PSYCHOMETRICS_REFUSED} ({0})")]
    PsychometricsRefused(&'static str),
    /// SQLite row count and JSONL line count disagree.
    #[error("sqlite/jsonl log diverged: sqlite={sqlite} jsonl={jsonl}")]
    LogDiverged { sqlite: u64, jsonl: u64 },
    /// Mode tag on the wire is not one of the four product surfaces.
    #[error("unknown attempt mode: {0}")]
    UnknownMode(String),
    /// A stored integer could not be converted back into the event type.
    #[error("stored integer out of range: {0}")]
    StoredInteger(String),
    /// SQLite failure (open, schema, bind, query).
    #[error("sqlite: {0}")]
    Sqlite(String),
    /// Filesystem failure on the JSONL mirror or the store directory.
    #[error("io: {0}")]
    Io(String),
    /// Event JSON could not be encoded or decoded.
    #[error("json: {0}")]
    Json(String),
}

impl From<rusqlite::Error> for AttemptError {
    fn from(e: rusqlite::Error) -> Self {
        Self::Sqlite(e.to_string())
    }
}

impl From<std::io::Error> for AttemptError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e.to_string())
    }
}

impl From<serde_json::Error> for AttemptError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e.to_string())
    }
}
