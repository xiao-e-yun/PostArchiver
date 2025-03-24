
use rusqlite::Connection;

use crate::{Migration, config::Config};

#[derive(Debug, Clone, Default)]
pub struct Bridge;

impl Migration for Bridge {
    const VERSION: &'static str = "v0.2";

    fn verify(&mut self, config: &Config) -> bool {
        let db_path = config.target.join("post-archiver.db");
        let conn = Connection::open(&db_path).expect("Failed to open database");
        !conn.query_row("SELECT count() FROM sqlite_master WHERE type='table' AND name='post_archiver_meta'",[],|row|row.get::<_,bool>(0)).unwrap()
    }

    fn upgrade(&mut self, config: &mut Config) {
        let db_path = config.target.join("post-archiver.db");
        let mut conn = Connection::open(&db_path).expect("Failed to open database");
        let tx = conn.transaction().unwrap();

        tx.execute_batch("
CREATE TABLE
    post_archiver_meta (version TEXT NOT NULL PRIMARY KEY);

CREATE TABLE
    features (
        name TEXT NOT NULL PRIMARY KEY,
        value INTEGER NOT NULL DEFAULT 0,
        extra JSON NOT NULL DEFAULT '{}'
    );

DROP TRIGGER update_post_thumb_on_file_meta_insert;
DROP TRIGGER update_post_thumb_on_file_meta_update;
DROP TRIGGER update_author_on_post_insert;
DROP TRIGGER update_author_on_post_update;

INSERT INTO post_archiver_meta (version) VALUES ('0.3.0');
        ").unwrap();

        tx.commit().expect("Failed to commit transaction");
    }
}
