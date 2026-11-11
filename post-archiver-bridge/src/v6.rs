use std::path::Path;

use log::info;
use rusqlite::Transaction;

use crate::MigrationDatabase;

/// v0.5 -> v0.6
///
/// - `tags.name` is no longer globally unique: general tags (`platform IS NULL`) are unique by
///   name, platform tags by `(platform, name)`.
/// - `author_aliases.source` is no longer globally unique, only per platform, and the table gets
///   an index on `target`.
/// - `file_metas` gets `UNIQUE (post, filename)`. Older importers could insert the same file twice,
///   so duplicates are merged first and every reference is pointed at the kept row.
/// - `posts_source_idx` is dropped (`posts.source` is already `UNIQUE`).
/// - Foreign keys are now enforced at runtime, so orphan rows left by earlier versions are removed
///   the same way `ON DELETE CASCADE` / `SET NULL` would have.
#[derive(Debug, Clone, Default)]
pub struct Bridge;

impl MigrationDatabase for Bridge {
    const VERSION: &'static str = "0.5";
    const SQL: &'static str = r#"
------------------------------------------------------------
-- Orphans (what ON DELETE CASCADE / SET NULL / SET DEFAULT would have done)
------------------------------------------------------------
DELETE FROM tags WHERE platform IS NOT NULL AND platform NOT IN (SELECT id FROM platforms);
DELETE FROM file_metas WHERE post NOT IN (SELECT id FROM posts);
DELETE FROM author_aliases WHERE target NOT IN (SELECT id FROM authors);
UPDATE OR IGNORE author_aliases SET platform = 0 WHERE platform NOT IN (SELECT id FROM platforms);
DELETE FROM author_aliases WHERE platform NOT IN (SELECT id FROM platforms);
UPDATE posts SET platform = NULL WHERE platform NOT IN (SELECT id FROM platforms);

------------------------------------------------------------
-- Merge duplicated file_metas: keep the newest row of each (post, filename)
------------------------------------------------------------
CREATE TEMP TABLE file_meta_remap (old INTEGER PRIMARY KEY, new INTEGER NOT NULL);

INSERT INTO file_meta_remap (old, new)
SELECT f.id, k.keep
FROM file_metas f
JOIN (
    SELECT post, filename, MAX(id) AS keep
    FROM file_metas
    GROUP BY post, filename
    HAVING COUNT(*) > 1
) k ON k.post = f.post AND k.filename = f.filename
WHERE f.id <> k.keep;

UPDATE posts SET thumb = (SELECT new FROM file_meta_remap WHERE old = posts.thumb)
WHERE thumb IN (SELECT old FROM file_meta_remap);
UPDATE authors SET thumb = (SELECT new FROM file_meta_remap WHERE old = authors.thumb)
WHERE thumb IN (SELECT old FROM file_meta_remap);
UPDATE collections SET thumb = (SELECT new FROM file_meta_remap WHERE old = collections.thumb)
WHERE thumb IN (SELECT old FROM file_meta_remap);

-- posts.content is a JSON array of text (string) and file ids (integer)
UPDATE posts SET content = (
    SELECT json_group_array(
        CASE j.type
            WHEN 'integer' THEN COALESCE((SELECT new FROM file_meta_remap WHERE old = j.value), j.value)
            WHEN 'object' THEN json(j.value)
            WHEN 'array' THEN json(j.value)
            ELSE j.value
        END
        ORDER BY j.key
    )
    FROM json_each(posts.content) j
)
WHERE EXISTS (
    SELECT 1 FROM json_each(posts.content) j
    WHERE j.type = 'integer' AND j.value IN (SELECT old FROM file_meta_remap)
);

DELETE FROM file_metas WHERE id IN (SELECT old FROM file_meta_remap);
DROP TABLE file_meta_remap;

------------------------------------------------------------
-- Dangling references to removed rows
------------------------------------------------------------
DELETE FROM author_posts
WHERE author NOT IN (SELECT id FROM authors) OR post NOT IN (SELECT id FROM posts);
DELETE FROM collection_posts
WHERE collection NOT IN (SELECT id FROM collections) OR post NOT IN (SELECT id FROM posts);
DELETE FROM post_tags
WHERE post NOT IN (SELECT id FROM posts) OR tag NOT IN (SELECT id FROM tags);

UPDATE posts SET thumb = NULL WHERE thumb NOT IN (SELECT id FROM file_metas);
UPDATE authors SET thumb = NULL WHERE thumb NOT IN (SELECT id FROM file_metas);
UPDATE collections SET thumb = NULL WHERE thumb NOT IN (SELECT id FROM file_metas);

------------------------------------------------------------
-- Schema
------------------------------------------------------------
DROP INDEX IF EXISTS posts_source_idx;
CREATE UNIQUE INDEX file_metas_idx ON file_metas (post, filename);

-- tags: drop UNIQUE on name
CREATE TABLE
    tags_new (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        platform INTEGER REFERENCES platforms (id) ON DELETE CASCADE
    );
INSERT INTO tags_new (id, name, platform) SELECT id, name, platform FROM tags;
DROP TABLE tags;
ALTER TABLE tags_new RENAME TO tags;

CREATE UNIQUE INDEX tags_idx ON tags (platform, name);
CREATE UNIQUE INDEX tags_general_idx ON tags (name) WHERE platform IS NULL;

-- author_aliases: drop UNIQUE on source
CREATE TABLE
    author_aliases_new (
        source TEXT NOT NULL,
        platform INTEGER NOT NULL DEFAULT 0,
        link TEXT,
        target INTEGER NOT NULL,
        FOREIGN KEY (platform) REFERENCES platforms (id) ON DELETE SET DEFAULT,
        FOREIGN KEY (target) REFERENCES authors (id) ON DELETE CASCADE,
        PRIMARY KEY (platform, source)
    );
INSERT INTO author_aliases_new (source, platform, link, target)
SELECT source, platform, link, target FROM author_aliases;
DROP TABLE author_aliases;
ALTER TABLE author_aliases_new RENAME TO author_aliases;

-- aliases are looked up by author (and cascade-deleted with it)
CREATE INDEX author_aliases_target_idx ON author_aliases (target);

UPDATE post_archiver_meta SET version = '0.6.0';
"#;

    fn upgrade(&mut self, _path: &Path, tx: &mut Transaction<'_>) {
        // Must be empty before foreign keys are turned back on; a panic here rolls back the
        // whole migration.
        let mut stmt = tx.prepare("PRAGMA foreign_key_check").unwrap();
        let violations = stmt
            .query_map([], |row| {
                Ok(format!(
                    "{} (rowid {:?}) -> {}",
                    row.get::<_, String>(0)?,
                    row.get::<_, Option<i64>>(1)?,
                    row.get::<_, String>(2)?
                ))
            })
            .unwrap()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        if !violations.is_empty() {
            panic!("foreign key violations after migration: {violations:#?}");
        }

        let ok: String = tx
            .query_row("PRAGMA integrity_check", [], |r| r.get(0))
            .unwrap();
        assert_eq!(ok, "ok", "integrity check failed");

        info!("Merged duplicated file metas and removed orphan rows");
    }
}
