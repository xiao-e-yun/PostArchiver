pub mod id;

#[cfg(feature = "importer")]
pub mod manager;

/// Relative path to the database file
pub const DATABASE_NAME: &str = "post-archiver.db";

/// Current version of the library.
///
/// It will be set to the version in `Cargo.toml` at build time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
