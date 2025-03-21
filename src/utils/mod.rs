pub mod rusqlite;

pub const DATABASE_NAME: &str = "post-archiver.db";
pub const DATABASE_VERSION: &str = env!("CARGO_PKG_VERSION");