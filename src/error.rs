//! Typed errors preserving serialization and output failures for the caller.

use thiserror::Error;

/// The crate's result type.
pub type Result<T> = std::result::Result<T, Error>;

/// Everything that can go wrong.
///
/// Inspect the source to distinguish invalid JSON serialization from a write
/// failure, then correct the value or retry with a writable destination.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Serializing a value or writing the JSON output failed.
    #[error("failed to render JSON: {0}")]
    Render(#[from] serde_json::Error),

    /// Reading or writing a file failed.
    #[error("failed to access `{path}`")]
    Io {
        /// The path that could not be accessed.
        path: String,
        /// Why.
        #[source]
        source: std::io::Error,
    },
}
