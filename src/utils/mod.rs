pub mod rusqlite;

pub const DATABASE_NAME: &str = "post-archiver.db";
pub const TEMPLATE_DATABASE_UP_SQL: &str = include_str!("template.up.sql");
pub const TEMPLATE_DATABASE_DOWN_SQL: &str = include_str!("template.down.sql");