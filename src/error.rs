//! Typed errors preserving serialization and output failures for the caller.

use thiserror::Error;

/// The crate's result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned while serializing JSON or writing it to the destination.
///
/// Inspect the source to distinguish invalid JSON serialization from a write
/// failure, then correct the value or retry with a writable destination.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Serializing a value or writing the JSON output failed.
    ///
    /// Inspect [`serde_json::Error::io_error_kind`] to identify write failures;
    /// the original serialization error is also available through source chaining.
    #[error("failed to render JSON: {0}")]
    Render(#[from] serde_json::Error),
}
