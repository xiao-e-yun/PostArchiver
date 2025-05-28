use crate::MigrationDatabase;

#[derive(Debug, Clone, Default)]
pub struct Bridge;

impl MigrationDatabase for Bridge {
    const VERSION: &'static str = "0.3";
    const SQL: &'static str = "
UPDATE post_archiver_meta SET version = '0.4.0';
ALTER TABLE author_alias RENAME TO author_aliases;

ALTER TABLE tags RENAME TO tags_old;
ALTER TABLE post_tags RENAME TO post_tags_old;

CREATE TABLE
    tags (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        category TEXT NOT NULL COLLATE NOCASE,
        name TEXT NOT NULL COLLATE NOCASE
        UNIQUE (category, name),
    );

CREATE INDEX tags_category_idx ON tags (category);
CREATE INDEX tags_idx ON tags (category, name);

CREATE TABLE
    post_tags (
        post INTEGER NOT NULL,
        tag INTEGER NOT NULL,
        PRIMARY KEY (post, tag),
        FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE,
        FOREIGN KEY (tag) REFERENCES tags (id) ON DELETE CASCADE
    );

CREATE INDEX post_tags_idx ON post_tags (tag);

-- Migrate old tags
INSERT INTO tags (id, category, name) SELECT (id, 'platform', name) FROM tags_old;
UPDATE tags SET category = 'general' WHERE name in ('r-18', 'free');

INSERT INTO post_tags (post, tag) SELECT (post, tag) FROM post_tags_old;

-- Remove old tables
DROP TABLE tags_old;
DROP TABLE post_tags_old;

VACUUM;
    ";
}
