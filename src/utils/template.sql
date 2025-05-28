------------------------------------------------------------
-- Base
------------------------------------------------------------
CREATE TABLE
    post_archiver_meta (version TEXT NOT NULL PRIMARY KEY);
CREATE TABLE
    features (
        name TEXT NOT NULL PRIMARY KEY,
        value INTEGER NOT NULL DEFAULT 0,
        extra JSON NOT NULL DEFAULT '{}'
    );

------------------------------------------------------------
-- Author
------------------------------------------------------------
CREATE TABLE
    authors (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL COLLATE NOCASE,
        links JSON NOT NULL DEFAULT '[]',
        thumb INTEGER,
        updated DATETIME NOT NULL DEFAULT "1970-01-01 00:00:00"
    );

-- Alias ---------------------------------------------------
CREATE TABLE
    author_aliases (
        -- source should be "site:author"
        source TEXT NOT NULL PRIMARY KEY,
        target INTEGER NOT NULL,
        FOREIGN KEY (target) REFERENCES authors (id) ON DELETE CASCADE
    );

------------------------------------------------------------
-- Post
------------------------------------------------------------
CREATE TABLE
    posts (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        author INTEGER NOT NULL,
        source TEXT,
        title TEXT NOT NULL,
        content JSON NOT NULL,
        thumb INTEGER,
        comments JSON NOT NULL DEFAULT '[]',
        updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        published DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
        FOREIGN KEY (author) REFERENCES authors (id) ON DELETE CASCADE
    );

CREATE INDEX posts_author_idx ON posts (author);

CREATE INDEX posts_source_idx ON posts (source);

CREATE INDEX posts_updated_idx ON posts (updated);

-- Tags ---------------------------------------------------
CREATE TABLE
    tags (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        category TEXT NOT NULL COLLATE NOCASE,
        name TEXT NOT NULL COLLATE NOCASE,
        UNIQUE (category, name)
    );

CREATE INDEX tags_category_idx ON tags (category);
CREATE INDEX tags_idx ON tags (category, name);

INSERT INTO tags (id, category, name) VALUES (0, 'platform', 'unknown');

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
        author INTEGER NOT NULL,
        post INTEGER NOT NULL,
        mime TEXT NOT NULL,
        extra JSON NOT NULL DEFAULT '{}',
        FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE
    );

CREATE INDEX file_metas_post_idx ON file_metas (post);
