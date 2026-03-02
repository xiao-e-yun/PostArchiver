//! Test helper functions providing direct SQL insert operations and query wrappers.

use std::collections::HashMap;

use chrono::{DateTime, Utc};
use rusqlite::params;
use serde_json::Value;

use crate::{
    manager::PostArchiverManager, query::Query, Alias, Author, AuthorId, Collection, CollectionId,
    FileMeta, FileMetaId, Platform, PlatformId, Post, PostId, Tag, TagId,
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
    m.get_post(id).unwrap().unwrap()
}

pub fn find_post(m: &PostArchiverManager, source: &str) -> Option<PostId> {
    m.find_post_by_source(source).unwrap()
}

pub fn find_post_with_updated(
    m: &PostArchiverManager,
    source: &str,
    updated: &DateTime<Utc>,
) -> Option<PostId> {
    use rusqlite::OptionalExtension;
    let mut stmt = m
        .conn()
        .prepare_cached("SELECT id FROM posts WHERE source = ? AND updated >= ?")
        .unwrap();
    stmt.query_row(params![source, updated], |row| row.get(0))
        .optional()
        .unwrap()
}

pub fn list_posts(m: &PostArchiverManager) -> Vec<Post> {
    m.posts().query().unwrap()
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
    m.get_author(id).unwrap().unwrap()
}

pub fn list_authors(m: &PostArchiverManager) -> Vec<Author> {
    m.authors().query().unwrap()
}

pub fn find_author(m: &PostArchiverManager, aliases: &[(&str, PlatformId)]) -> Option<AuthorId> {
    for (source, platform) in aliases {
        if let Some(id) = m.find_author_by_alias(source, *platform).unwrap() {
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
    m.bind(author).list_aliases().unwrap()
}

pub fn list_author_posts(m: &PostArchiverManager, author: AuthorId) -> Vec<Post> {
    m.bind(author)
        .list_posts()
        .unwrap()
        .into_iter()
        .filter_map(|id| m.get_post(id).unwrap())
        .collect()
}

pub fn list_post_authors(m: &PostArchiverManager, post: PostId) -> Vec<Author> {
    m.bind(post)
        .list_authors()
        .unwrap()
        .into_iter()
        .filter_map(|id| m.get_author(id).unwrap())
        .collect()
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
    m.get_platform(id).unwrap().unwrap()
}

pub fn find_platform(m: &PostArchiverManager, name: &str) -> Option<PlatformId> {
    m.find_platform(name).unwrap()
}

pub fn list_platforms(m: &PostArchiverManager) -> Vec<Platform> {
    m.platforms().query().unwrap()
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
    m.get_tag(id).unwrap()
}

pub fn find_tag(
    m: &PostArchiverManager,
    name: &str,
    platform: Option<PlatformId>,
) -> Option<TagId> {
    m.find_tag(name, platform).unwrap()
}

pub fn list_tags(m: &PostArchiverManager) -> Vec<Tag> {
    m.tags().query().unwrap()
}

pub fn list_post_tags(m: &PostArchiverManager, post: PostId) -> Vec<Tag> {
    m.bind(post)
        .list_tags()
        .unwrap()
        .into_iter()
        .filter_map(|id| m.get_tag(id).unwrap())
        .collect()
}

pub fn list_tag_posts(m: &PostArchiverManager, tag: TagId) -> Vec<Post> {
    let mut q = m.posts();
    q.tags.insert(tag);
    q.query().unwrap()
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
    m.get_collection(id).unwrap()
}

pub fn find_collection(m: &PostArchiverManager, source: &str) -> Option<CollectionId> {
    m.find_collection_by_source(source).unwrap()
}

pub fn list_collections(m: &PostArchiverManager) -> Vec<Collection> {
    m.collections().query().unwrap()
}

pub fn list_post_collections(m: &PostArchiverManager, post: PostId) -> Vec<Collection> {
    m.bind(post)
        .list_collections()
        .unwrap()
        .into_iter()
        .filter_map(|id| m.get_collection(id).unwrap())
        .collect()
}

pub fn list_collection_posts(m: &PostArchiverManager, collection: CollectionId) -> Vec<Post> {
    let mut q = m.posts();
    q.collections.insert(collection);
    q.query().unwrap()
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
    m.get_file_meta(id).unwrap().unwrap()
}

pub fn find_file_meta(m: &PostArchiverManager, post: PostId, filename: &str) -> Option<FileMetaId> {
    m.find_file_meta(post, filename).unwrap()
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
