use std::{fs, path::Path};

use log::{debug, info};
use post_archiver_latest::POSTS_PRE_CHUNK;
use rusqlite::Transaction;

use crate::MigrationDatabase;

#[derive(Debug, Clone, Default)]
pub struct Bridge;

impl MigrationDatabase for Bridge {
    const VERSION: &'static str = "0.3";
    const SQL: &'static str = "
-- Rename the existing table to preserve data
ALTER TABLE authors RENAME TO authors_old;
ALTER TABLE author_alias RENAME TO author_aliases_old;
ALTER TABLE posts RENAME TO posts_old;
ALTER TABLE tags RENAME TO tags_old;
ALTER TABLE post_tags RENAME TO post_tags_old;
ALTER TABLE file_metas RENAME TO file_metas_old;

-- Drop indexes that will be recreated
DROP INDEX IF EXISTS posts_source_idx;
DROP INDEX IF EXISTS posts_updated_idx;
DROP INDEX IF EXISTS posts_platform_idx;

-- Create and update new version of the database schemas

------------------------------------------------------------
-- Author
------------------------------------------------------------
CREATE TABLE
    authors (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        thumb INTEGER,
        updated DATETIME NOT NULL DEFAULT '1970-01-01 00:00:00',
        FOREIGN KEY (thumb) REFERENCES file_metas (id) ON DELETE SET NULL
    );

-- Alias ---------------------------------------------------
CREATE TABLE
    author_aliases (
        source TEXT UNIQUE NOT NULL,
        platform INTEGER NOT NULL DEFAULT 0,
        link TEXT,
        target INTEGER NOT NULL,
        FOREIGN KEY (platform) REFERENCES platforms (id) ON DELETE SET DEFAULT,
        FOREIGN KEY (target) REFERENCES authors (id) ON DELETE CASCADE,
        PRIMARY KEY (platform, source)
    );

-- Post ---------------------------------------------------
CREATE TABLE
    author_posts (
        author INTEGER NOT NULL,
        post INTEGER NOT NULL,
        PRIMARY KEY (author, post),
        FOREIGN KEY (author) REFERENCES authors (id) ON DELETE CASCADE,
        FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE
    );

CREATE INDEX author_posts_post_idx ON author_posts (post);

------------------------------------------------------------
-- Post
------------------------------------------------------------
CREATE TABLE
    posts (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        source TEXT UNIQUE,
        platform INTEGER,
        title TEXT NOT NULL,
        thumb INTEGER,
        content JSON NOT NULL DEFAULT '[]',
        comments JSON NOT NULL DEFAULT '[]',
        published DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (platform) REFERENCES platforms (id) ON DELETE SET NULL,
        FOREIGN KEY (thumb) REFERENCES file_metas (id) ON DELETE SET NULL
    );

CREATE INDEX posts_source_idx ON posts (source);
CREATE INDEX posts_updated_idx ON posts (updated);
CREATE INDEX posts_platform_idx ON posts (platform);

-- platform -----------------------------------------------
CREATE TABLE
    platforms (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE COLLATE NOCASE
    );

CREATE INDEX platforms_name_idx ON platforms (name);

INSERT INTO platforms (id, name) VALUES (0, 'unknown');

-- collection -----------------------------------------------
CREATE TABLE
    collections (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL,
        source TEXT UNIQUE,
        thumb INTEGER REFERENCES file_metas (id) ON DELETE SET NULL
    );

CREATE INDEX collections_name_idx ON collections (name);
CREATE INDEX collections_source_idx ON collections (source);

CREATE TABLE
    collection_posts (
        collection INTEGER NOT NULL,
        post INTEGER NOT NULL,
        PRIMARY KEY (collection, post),
        FOREIGN KEY (collection) REFERENCES collections (id) ON DELETE CASCADE,
        FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE
    );

CREATE INDEX collection_posts_post_idx ON collection_posts (post);

-- Tags ---------------------------------------------------
CREATE TABLE
    tags (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL UNIQUE ,
        platform INTEGER REFERENCES platforms (id) ON DELETE CASCADE
    );

CREATE UNIQUE INDEX tags_idx ON tags (platform, name);

CREATE TABLE
    post_tags (
        post INTEGER NOT NULL,
        tag INTEGER NOT NULL,
        PRIMARY KEY (post, tag),
        FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE,
        FOREIGN KEY (tag) REFERENCES tags (id) ON DELETE CASCADE
    );

CREATE INDEX post_tags_idx ON post_tags (tag);

------------------------------------------------------------
-- File Meta
------------------------------------------------------------
CREATE TABLE
    file_metas (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        filename TEXT NOT NULL,
        post INTEGER NOT NULL,
        mime TEXT NOT NULL,
        extra JSON NOT NULL DEFAULT '{}',
        FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE
    );

-- Clone old data into new tables
INSERT INTO platforms (name)
    SELECT name FROM tags_old WHERE name IN ('fanbox-dl', 'fanbox', 'patreon');

INSERT INTO platforms (name)
    SELECT 'pixiv' AS name FROM tags_old WHERE name IN ('fanbox');

CREATE TEMPORARY TABLE platforms_raw AS
    SELECT platforms.id, platforms.name, tags_old.id AS raw_id
    FROM platforms
    JOIN tags_old ON tags_old.name = platforms.name
    WHERE tags_old.name IN ('fanbox-dl', 'fanbox', 'patreon', 'pixiv');

INSERT INTO authors (id, name, thumb, updated)
    SELECT id, name, thumb, updated FROM authors_old;

INSERT INTO author_aliases (source, platform, link, target)
    SELECT
      substr(aliases.source, instr(aliases.source, ':') + 1) AS source,
      platforms.id,
      (
        SELECT json_extract(links.value, '$.url')
        FROM authors_old, json_each(authors_old.links) AS links
        WHERE json_extract(links.value, '$.name') = substr(aliases.source, 1, instr(aliases.source, ':') - 1)
      ) AS link,
      aliases.target
  FROM author_aliases_old AS aliases
  JOIN platforms ON platforms.name = substr(aliases.source, 1, instr(aliases.source, ':') - 1);

INSERT INTO author_posts (author, post)
    SELECT author, id AS post FROM posts_old;

INSERT INTO posts (id, source, platform, title, thumb, content, comments, published, updated)
    SELECT
        posts.id,
        posts.source,
        0 AS platform, -- Default to 0 for unknown platform
        posts.title,
        posts.thumb,
        posts.content,
        posts.comments,
        posts.published,
        posts.updated
    FROM posts_old as posts;

UPDATE posts SET platform = platforms.id
    FROM post_tags_old AS post_tags
    JOIN platforms_raw AS platforms ON post_tags.tag = platforms.raw_id
    WHERE posts.id = post_tags.post;

INSERT INTO tags (name)
    SELECT
        name
    FROM tags_old
    WHERE name IN ('free', 'r-18');

CREATE TEMPORARY TABLE tags_raw AS
    SELECT tags.id, tags_old.id AS raw_id
    FROM tags
    JOIN tags_old ON tags_old.name = tags.name;

INSERT INTO post_tags (post, tag)
    SELECT
        post_tags.post,
        tags_raw.id
    FROM post_tags_old AS post_tags
    JOIN tags_raw ON post_tags.tag = tags_raw.raw_id;

INSERT INTO file_metas (id, filename, post, mime, extra)
    SELECT
        file_metas_old.id,
        file_metas_old.filename,
        file_metas_old.post,
        file_metas_old.mime,
        file_metas_old.extra
    FROM file_metas_old;

-- Clean up old tables
DROP TABLE authors_old;
DROP TABLE author_aliases_old;
DROP TABLE posts_old;
DROP TABLE tags_old;
DROP TABLE post_tags_old;
DROP TABLE file_metas_old;
    ";

    fn upgrade(&mut self, path: &Path, tx: &mut Transaction<'_>) {
        info!("Upgrading file structure for v{}", Self::VERSION);

        fs::create_dir(path.join("v3_old")).expect("Failed to create temp old directory");

        debug!("Moving old post directories to v3_old");
        for entry in path
            .read_dir()
            .expect("Failed to read directory")
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                continue;
            }
            if entry
                .file_name()
                .to_string_lossy()
                .parse::<usize>()
                .is_err()
            {
                continue;
            }

            let temp_path = path.join("v3_old").join(entry.file_name());

            fs::rename(entry.path(), &temp_path)
                .expect("Failed to rename post directory to authors");
        }

        let mut stmt = tx.prepare("SELECT posts.id, author_posts.author FROM posts JOIN author_posts ON posts.id == author_posts.post").unwrap();
        let mut posts = stmt.query([]).unwrap();

        while let Some(row) = posts.next().unwrap() {
            let post_id: u32 = row.get(0).unwrap();
            let author_id: u32 = row.get(1).unwrap();

            // Move the post to the author's directory
            let chunk = post_id / POSTS_PRE_CHUNK;
            let index = post_id % POSTS_PRE_CHUNK;
            let source = path
                .join("v3_old")
                .join(author_id.to_string())
                .join(post_id.to_string());
            let target = path.join(chunk.to_string()).join(index.to_string());

            if source.exists() {
                debug!(
                    "Moving post {} from {} to {}",
                    post_id,
                    source.display(),
                    target.display()
                );
                fs::create_dir_all(target.parent().expect("Failed to get parent directory"))
                    .expect("Failed to create target directory");

                fs::rename(source, target).expect("Failed to move post directory");
            }
        }

        // Remove the old directory if is empty
        debug!("Cleaning up old directories");
        for entry in path
            .join("v3_old")
            .read_dir()
            .expect("Failed to read directory")
            .filter_map(Result::ok)
        {
            if !entry.file_type().is_ok_and(|ft| ft.is_dir()) {
                continue;
            }
            if entry
                .file_name()
                .to_string_lossy()
                .parse::<usize>()
                .is_err()
            {
                continue;
            }

            // Remove the old authors directory
            fs::remove_dir(entry.path()).ok();
        }

        fs::remove_dir(path.join("v3_old")).ok();
    }
}
