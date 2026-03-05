use std::path::Path;

use rusqlite::Transaction;

use crate::MigrationDatabase;

#[derive(Debug, Clone, Default)]
pub struct Bridge;

impl MigrationDatabase for Bridge {
    const VERSION: &'static str = "0.4";
    const SQL: &'static str = "
    DROP TABLE IF EXISTS features;
    UPDATE post_archiver_meta SET version = '0.5.0';
    ";

    fn upgrade(&mut self, _path: &Path, _tx: &mut Transaction<'_>) {}
}
