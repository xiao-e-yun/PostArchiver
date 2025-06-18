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
        name TEXT NOT NULL,
        thumb INTEGER,
        updated DATETIME NOT NULL DEFAULT "1970-01-01 00:00:00",
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

CREATE TABLE 
    post_platform_tags (
        post INTEGER NOT NULL,
        tag INTEGER NOT NULL,
        PRIMARY KEY (post, tag),
        FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE,
        FOREIGN KEY (tag) REFERENCES platform_tags (id) ON DELETE CASCADE
    );

CREATE INDEX post_platform_tags_idx ON post_platform_tags (tag);

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
