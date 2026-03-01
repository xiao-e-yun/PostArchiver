use std::{fs, path::Path};

use log::{debug, info};
use post_archiver_latest::POSTS_PRE_CHUNK;
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
}
