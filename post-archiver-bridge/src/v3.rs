use crate::MigrationDatabase;

#[derive(Debug, Clone, Default)]
pub struct BridgeV0;

impl MigrationDatabase for BridgeV0 {
    const VERSION: &'static str = "0.3";
    const SQL: &'static str = "
UPDATE post_archiver_meta SET version = '0.4.0';
ALTER TABLE author_alias RENAME TO author_aliases;
    ";
}
