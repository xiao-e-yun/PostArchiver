------------------------------------------------------------
-- Author System
------------------------------------------------------------
CREATE TABLE
    authors (
        id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
        name TEXT NOT NULL COLLATE NOCASE,
        links JSON NOT NULL DEFAULT '[]',
        thumb INTEGER,
        updated DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
    );

-- Alias ---------------------------------------------------
CREATE TABLE
    author_alias (
        -- source should be "site:author"
        source TEXT NOT NULL PRIMARY KEY,
        target INTEGER NOT NULL,
        FOREIGN KEY (target) REFERENCES authors (id) ON DELETE CASCADE
    );

------------------------------------------------------------
-- Post System
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
        name TEXT NOT NULL UNIQUE COLLATE NOCASE
    );

INSERT INTO
    tags (id, name)
VALUES
    (0, 'unknown');

CREATE TABLE post_tags (
    post INTEGER NOT NULL,
    tag INTEGER NOT NULL,
    PRIMARY KEY (post, tag),
    FOREIGN KEY (post) REFERENCES posts (id) ON DELETE CASCADE,
    FOREIGN KEY (tag) REFERENCES tags (id) ON DELETE CASCADE
);

------------------------------------------------------------
-- File Meta System
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

------------------------------------------------------------
-- Thumb System
------------------------------------------------------------
-- update post thumb 
CREATE TRIGGER update_post_thumb_on_file_meta_insert AFTER INSERT ON file_metas BEGIN
UPDATE posts
SET
    thumb = NEW.id
WHERE
    id = NEW.post
    AND NEW.mime LIKE 'image/%';

END;

CREATE TRIGGER update_post_thumb_on_file_meta_update AFTER
UPDATE ON file_metas BEGIN
UPDATE posts
SET
    thumb = NEW.id
WHERE
    id = NEW.post
    AND NEW.mime LIKE 'image/%';

END;

-- update author updatedTime and thumb
CREATE TRIGGER update_author_on_post_insert AFTER INSERT ON posts BEGIN
UPDATE authors
SET
    updated = CURRENT_TIMESTAMP
WHERE
    id = NEW.author;

UPDATE authors
SET
    thumb = NEW.thumb
WHERE
    id = NEW.author
    AND NEW.thumb IS NOT NULL;

END;

CREATE TRIGGER update_author_on_post_update AFTER
UPDATE ON posts BEGIN
UPDATE authors
SET
    updated = CURRENT_TIMESTAMP
WHERE
    id = NEW.author;

UPDATE authors
SET
    thumb = NEW.thumb
WHERE
    id = NEW.author
    AND NEW.thumb IS NOT NULL;

END;