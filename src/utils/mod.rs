pub mod rusqlite;

pub const DATABASE_NAME: &str = "post-archiver.db";
pub const TEMPLATE_DATABASE_SQL: &str = include_str!("template.up.sql");