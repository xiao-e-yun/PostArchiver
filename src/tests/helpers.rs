//! Test helper functions providing direct SQL insert/query operations.
//!
//! These functions are needed because the manager module now only exposes
//! update/delete/relation operations through `Binded`. Insert and query
//! methods will eventually live in dedicated importer/query modules.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::{
    manager::PostArchiverManager, utils::macros::AsTable, Alias, Author, AuthorId, Collection,
    CollectionId, FileMeta, FileMetaId, Platform, PlatformId, Post, PostId, Tag, TagId,
};

// ── Post helpers ──────────────────────────────────────────────

pub fn add_post(
    m: &PostArchiverManager,
    title: String,
    source: Option<String>,
    platform: Option<PlatformId>,
    published: Option<DateTime<Utc>>,
    updated: Option<DateTime<Utc>>,
) -> PostId {
    let now = Utc::now();
    let mut stmt = m
        .conn()
        .prepare_cached(
            "INSERT INTO posts (title, source, platform, published, updated) VALUES (?, ?, ?, ?, ?) RETURNING id",
        )
        .unwrap();
    stmt.query_row(
        params![
            title,
            source,
            platform,
            published.unwrap_or(now),
            updated.unwrap_or(now)
        ],
        |row| row.get(0),
    )
    .unwrap()
}

pub fn get_post(m: &PostArchiverManager, id: PostId) -> Post {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM posts WHERE id = ?")
        .unwrap();
    stmt.query_row([id], Post::from_row).unwrap()
}

pub fn find_post(m: &PostArchiverManager, source: &str) -> Option<PostId> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT id FROM posts WHERE source = ?")
        .unwrap();
    stmt.query_row([source], |row| row.get(0))
        .optional()
        .unwrap()
}

pub fn find_post_with_updated(
    m: &PostArchiverManager,
    source: &str,
    updated: &DateTime<Utc>,
) -> Option<PostId> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT id FROM posts WHERE source = ? AND updated >= ?")
        .unwrap();
    stmt.query_row(params![source, updated], |row| row.get(0))
        .optional()
        .unwrap()
}

pub fn list_posts(m: &PostArchiverManager) -> Vec<Post> {
    let mut stmt = m.conn().prepare_cached("SELECT * FROM posts").unwrap();
    let rows = stmt.query_map([], Post::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

// ── Author helpers ────────────────────────────────────────────

pub fn add_author(
    m: &PostArchiverManager,
    name: String,
    updated: Option<DateTime<Utc>>,
) -> AuthorId {
    let mut stmt = m
        .conn()
        .prepare_cached("INSERT INTO authors (name, updated) VALUES (?, ?) RETURNING id")
        .unwrap();
    stmt.query_row(params![name, updated.unwrap_or_else(Utc::now)], |row| {
        row.get(0)
    })
    .unwrap()
}

pub fn get_author(m: &PostArchiverManager, id: AuthorId) -> Author {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM authors WHERE id = ?")
        .unwrap();
    stmt.query_row([id], |row| {
        Ok(Author {
            id: row.get("id")?,
            name: row.get("name")?,
            thumb: row.get("thumb")?,
            updated: row.get("updated")?,
        })
    })
    .unwrap()
}

pub fn list_authors(m: &PostArchiverManager) -> Vec<Author> {
    let mut stmt = m.conn().prepare_cached("SELECT * FROM authors").unwrap();
    let rows = stmt
        .query_map([], |row| {
            Ok(Author {
                id: row.get("id")?,
                name: row.get("name")?,
                thumb: row.get("thumb")?,
                updated: row.get("updated")?,
            })
        })
        .unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

pub fn find_author(m: &PostArchiverManager, aliases: &[(&str, PlatformId)]) -> Option<AuthorId> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT target FROM author_aliases WHERE platform = ? AND source = ?")
        .unwrap();
    for (source, platform) in aliases {
        if let Some(id) = stmt
            .query_row(params![platform, source], |row| row.get(0))
            .optional()
            .unwrap()
        {
            return Some(id);
        }
    }
    None
}

pub fn add_author_aliases(
    m: &PostArchiverManager,
    author: AuthorId,
    aliases: Vec<(String, PlatformId, Option<String>)>,
) {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "INSERT OR REPLACE INTO author_aliases (target, source, platform, link) VALUES (?, ?, ?, ?)",
        )
        .unwrap();
    for (source, platform, link) in aliases {
        stmt.execute(params![author, source, platform, link])
            .unwrap();
    }
}

pub fn list_author_aliases(m: &PostArchiverManager, author: AuthorId) -> Vec<Alias> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")
        .unwrap();
    let rows = stmt.query_map([author], Alias::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

pub fn list_author_posts(m: &PostArchiverManager, author: AuthorId) -> Vec<Post> {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "SELECT posts.* FROM posts INNER JOIN author_posts ON author_posts.post = posts.id WHERE author_posts.author = ?",
        )
        .unwrap();
    let rows = stmt.query_map([author], Post::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

pub fn list_post_authors(m: &PostArchiverManager, post: PostId) -> Vec<Author> {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "SELECT authors.* FROM authors INNER JOIN author_posts ON author_posts.author = authors.id WHERE author_posts.post = ?",
        )
        .unwrap();
    let rows = stmt
        .query_map([post], |row| {
            Ok(Author {
                id: row.get("id")?,
                name: row.get("name")?,
                thumb: row.get("thumb")?,
                updated: row.get("updated")?,
            })
        })
        .unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

// ── Platform helpers ──────────────────────────────────────────

pub fn add_platform(m: &PostArchiverManager, name: String) -> PlatformId {
    let mut stmt = m
        .conn()
        .prepare_cached("INSERT INTO platforms (name) VALUES (?) RETURNING id")
        .unwrap();
    stmt.query_row([&name], |row| row.get(0)).unwrap()
}

pub fn get_platform(m: &PostArchiverManager, id: PlatformId) -> Platform {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM platforms WHERE id = ?")
        .unwrap();
    stmt.query_row([id], Platform::from_row).unwrap()
}

pub fn find_platform(m: &PostArchiverManager, name: &str) -> Option<PlatformId> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT id FROM platforms WHERE name = ?")
        .unwrap();
    stmt.query_row([name], |row| row.get(0)).optional().unwrap()
}

pub fn list_platforms(m: &PostArchiverManager) -> Vec<Platform> {
    let mut stmt = m.conn().prepare_cached("SELECT * FROM platforms").unwrap();
    let rows = stmt.query_map([], Platform::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

// ── Tag helpers ───────────────────────────────────────────────

pub fn add_tag(m: &PostArchiverManager, name: String, platform: Option<PlatformId>) -> TagId {
    let mut stmt = m
        .conn()
        .prepare_cached("INSERT INTO tags (name, platform) VALUES (?, ?) RETURNING id")
        .unwrap();
    stmt.query_row(params![name, platform], |row| row.get(0))
        .unwrap()
}

pub fn get_tag(m: &PostArchiverManager, id: TagId) -> Option<Tag> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM tags WHERE id = ?")
        .unwrap();
    stmt.query_row([id], Tag::from_row).optional().unwrap()
}

pub fn find_tag(
    m: &PostArchiverManager,
    name: &str,
    platform: Option<PlatformId>,
) -> Option<TagId> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT id FROM tags WHERE platform IS ? AND name = ?")
        .unwrap();
    stmt.query_row(params![platform, name], |row| row.get(0))
        .optional()
        .unwrap()
}

pub fn list_tags(m: &PostArchiverManager) -> Vec<Tag> {
    let mut stmt = m.conn().prepare_cached("SELECT * FROM tags").unwrap();
    let rows = stmt.query_map([], Tag::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

pub fn list_post_tags(m: &PostArchiverManager, post: PostId) -> Vec<Tag> {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "SELECT tags.* FROM tags INNER JOIN post_tags ON post_tags.tag = tags.id WHERE post_tags.post = ?",
        )
        .unwrap();
    let rows = stmt.query_map([post], Tag::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

pub fn list_tag_posts(m: &PostArchiverManager, tag: TagId) -> Vec<Post> {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "SELECT posts.* FROM posts INNER JOIN post_tags ON post_tags.post = posts.id WHERE post_tags.tag = ?",
        )
        .unwrap();
    let rows = stmt.query_map([tag], Post::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

// ── Collection helpers ────────────────────────────────────────

pub fn add_collection(
    m: &PostArchiverManager,
    name: String,
    source: Option<String>,
    thumb: Option<FileMetaId>,
) -> CollectionId {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "INSERT INTO collections (name, source, thumb) VALUES (?, ?, ?) RETURNING id",
        )
        .unwrap();
    stmt.query_row(params![name, source, thumb], |row| row.get(0))
        .unwrap()
}

pub fn get_collection(m: &PostArchiverManager, id: CollectionId) -> Option<Collection> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM collections WHERE id = ?")
        .unwrap();
    stmt.query_row([id], Collection::from_row)
        .optional()
        .unwrap()
}

pub fn find_collection(m: &PostArchiverManager, source: &str) -> Option<CollectionId> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT id FROM collections WHERE source = ?")
        .unwrap();
    stmt.query_row([source], |row| row.get(0))
        .optional()
        .unwrap()
}

pub fn list_collections(m: &PostArchiverManager) -> Vec<Collection> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM collections")
        .unwrap();
    let rows = stmt.query_map([], Collection::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

pub fn list_post_collections(m: &PostArchiverManager, post: PostId) -> Vec<Collection> {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "SELECT collections.* FROM collections INNER JOIN collection_posts ON collection_posts.collection = collections.id WHERE collection_posts.post = ?",
        )
        .unwrap();
    let rows = stmt.query_map([post], Collection::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

pub fn list_collection_posts(m: &PostArchiverManager, collection: CollectionId) -> Vec<Post> {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "SELECT posts.* FROM posts INNER JOIN collection_posts ON collection_posts.post = posts.id WHERE collection_posts.collection = ?",
        )
        .unwrap();
    let rows = stmt.query_map([collection], Post::from_row).unwrap();
    rows.collect::<Result<Vec<_>, _>>().unwrap()
}

// ── FileMeta helpers ──────────────────────────────────────────

pub fn add_file_meta(
    m: &PostArchiverManager,
    post: PostId,
    filename: String,
    mime: String,
    extra: HashMap<String, Value>,
) -> FileMetaId {
    let mut stmt = m
        .conn()
        .prepare_cached(
            "INSERT INTO file_metas (post, filename, mime, extra) VALUES (?, ?, ?, ?) RETURNING id",
        )
        .unwrap();
    stmt.query_row(
        params![post, filename, mime, serde_json::to_string(&extra).unwrap()],
        |row| row.get(0),
    )
    .unwrap()
}

pub fn get_file_meta(m: &PostArchiverManager, id: FileMetaId) -> FileMeta {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT * FROM file_metas WHERE id = ?")
        .unwrap();
    stmt.query_row([id], FileMeta::from_row).unwrap()
}

pub fn find_file_meta(m: &PostArchiverManager, post: PostId, filename: &str) -> Option<FileMetaId> {
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT id FROM file_metas WHERE post = ? AND filename = ?")
        .unwrap();
    stmt.query_row(params![post, filename], |row| row.get(0))
        .optional()
        .unwrap()
}

// ── Relationship helpers (add) ────────────────────────────────

pub fn add_post_authors(m: &PostArchiverManager, post: PostId, authors: &[AuthorId]) {
    let mut stmt = m
        .conn()
        .prepare_cached("INSERT OR IGNORE INTO author_posts (author, post) VALUES (?, ?)")
        .unwrap();
    for author in authors {
        stmt.execute(params![author, post]).unwrap();
    }
}

pub fn add_post_tags(m: &PostArchiverManager, post: PostId, tags: &[TagId]) {
    let mut stmt = m
        .conn()
        .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")
        .unwrap();
    for tag in tags {
        stmt.execute(params![post, tag]).unwrap();
    }
}

pub fn add_post_collections(m: &PostArchiverManager, post: PostId, collections: &[CollectionId]) {
    let mut stmt = m
        .conn()
        .prepare_cached("INSERT OR IGNORE INTO collection_posts (collection, post) VALUES (?, ?)")
        .unwrap();
    for collection in collections {
        stmt.execute(params![collection, post]).unwrap();
    }
}
