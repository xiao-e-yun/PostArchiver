//! Error types for the post archiver system.

/// The main error type for post archiver operations.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// A SQLite database error.
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),

    /// A filesystem I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// The database already exists at the given path.
    #[error("database already exists")]
    DatabaseAlreadyExists,

    /// The database version does not match the expected version.
    #[error("database version mismatch: current {current}, expected {expected}")]
    VersionMismatch { current: String, expected: String },
}

/// A specialized `Result` type for post archiver operations.
pub type Result<T> = std::result::Result<T, Error>;
