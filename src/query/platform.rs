//! Platform query builder and related point-query helpers.

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    Platform, PlatformId, Post, Tag,
};

// ── Builder ───────────────────────────────────────────────────────────────────

/// Query builder for platforms.  Obtained via [`PostArchiverManager::platforms()`].
///
/// Platforms are few, so this builder is intentionally simple — no pagination or
/// typestate is provided.  `.query()` always returns `Vec<Platform>`.
#[derive(Debug)]
pub struct PlatformQuery<'a, C> {
    manager: &'a PostArchiverManager<C>,
}

impl<'a, C> PlatformQuery<'a, C> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        PlatformQuery { manager }
    }
}

impl<C: PostArchiverConnection> PlatformQuery<'_, C> {
    /// Return all platforms ordered by name.
    pub fn query(self) -> Result<Vec<Platform>, rusqlite::Error> {
        let mut stmt = self
            .manager
            .conn()
            .prepare_cached("SELECT * FROM platforms ORDER BY name ASC")?;
        let rows = stmt.query_map([], Platform::from_row)?;
        rows.collect()
    }
}

// ── Point-query helpers on PostArchiverManager ────────────────────────────────

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the platform query builder.
    pub fn platforms(&self) -> PlatformQuery<'_, C> {
        PlatformQuery::new(self)
    }

    /// Fetch a single platform by primary key.
    pub fn get_platform(&self, id: PlatformId) -> Result<Option<Platform>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM platforms WHERE id = ?")?;
        stmt.query_row([id], Platform::from_row).optional()
    }

    /// Find a platform ID by name (exact match, case-sensitive).
    pub fn find_platform(&self, name: &str) -> Result<Option<PlatformId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM platforms WHERE name = ?")?;
        stmt.query_row([name], |row| row.get(0)).optional()
    }

    /// Fetch all posts on a platform (full entities).
    pub fn list_platform_posts(&self, platform: PlatformId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE platform = ?")?;
        let rows = stmt.query_map([platform], Post::from_row)?;
        rows.collect()
    }

    /// Fetch all tags belonging to a platform (full entities).
    pub fn list_platform_tags(&self, platform: PlatformId) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM tags WHERE platform = ?")?;
        let rows = stmt.query_map([platform], Tag::from_row)?;
        rows.collect()
    }
}
