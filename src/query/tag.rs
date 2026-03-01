//! Tag query builder and related point-query helpers.

use std::marker::PhantomData;

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    PlatformId, Tag, TagId,
};

use super::{NoTotal, PageResult, SortDir, WithTotal};

// ── Builder ───────────────────────────────────────────────────────────────────

/// Fluent query builder for tags.  Obtained via [`PostArchiverManager::tags()`].
///
/// The `platform` filter accepts `Option<PlatformId>`:
/// - `Some(id)` filters tags belonging to that platform.
/// - `None` filters tags with no platform (`platform IS NULL`).
/// Multiple calls are combined with OR.
#[derive(Debug)]
pub struct TagQuery<'a, C, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    /// Each entry is `Some(platform)` = has platform, `None` = no platform.
    platforms: Vec<Option<PlatformId>>,
    name_contains: Option<String>,
    limit: u64,
    page: u64,
    sort_dir: SortDir,
    _t: PhantomData<T>,
}

impl<'a, C, T> TagQuery<'a, C, T> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        TagQuery {
            manager,
            platforms: Vec::new(),
            name_contains: None,
            limit: 50,
            page: 0,
            sort_dir: SortDir::default(),
            _t: PhantomData,
        }
    }

    /// Filter tags by platform. Pass `None` to include tags with no platform.
    pub fn platform(mut self, platform: Option<PlatformId>) -> Self {
        self.platforms.push(platform);
        self
    }

    /// Filter tags whose name contains `keyword` (case-insensitive `LIKE`).
    pub fn name_contains(mut self, keyword: impl Into<String>) -> Self {
        self.name_contains = Some(keyword.into());
        self
    }

    /// Set pagination.
    pub fn pagination(mut self, limit: u64, page: u64) -> Self {
        self.limit = limit;
        self.page = page;
        self
    }

    /// Set sort direction (tags sort by `name`).
    pub fn sort_dir(mut self, dir: SortDir) -> Self {
        self.sort_dir = dir;
        self
    }

    /// Transition: include total row count.
    pub fn with_total(self) -> TagQuery<'a, C, WithTotal> {
        TagQuery {
            manager: self.manager,
            platforms: self.platforms,
            name_contains: self.name_contains,
            limit: self.limit,
            page: self.page,
            sort_dir: self.sort_dir,
            _t: PhantomData,
        }
    }
}

// ── Internal SQL helpers ──────────────────────────────────────────────────────

type BoxParam = Box<dyn rusqlite::types::ToSql>;

impl<C: PostArchiverConnection, T> TagQuery<'_, C, T> {
    fn build_where(&self) -> (String, Vec<BoxParam>) {
        let mut wheres: Vec<String> = Vec::new();
        let mut params: Vec<BoxParam> = Vec::new();

        if !self.platforms.is_empty() {
            // Build OR-connected conditions for each entry
            let conds: Vec<String> = self
                .platforms
                .iter()
                .map(|p| {
                    if p.is_some() {
                        "platform = ?".to_string()
                    } else {
                        "platform IS NULL".to_string()
                    }
                })
                .collect();
            wheres.push(format!("({})", conds.join(" OR ")));
            for p in &self.platforms {
                if let Some(id) = p {
                    params.push(Box::new(*id));
                }
                // NULL conditions don't need a param
            }
        }

        if let Some(ref kw) = self.name_contains {
            wheres.push("name LIKE '%' || ? || '%'".to_string());
            params.push(Box::new(kw.clone()));
        }

        let clause = if wheres.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", wheres.join(" AND "))
        };

        (clause, params)
    }

    fn fetch_tags(&self) -> Result<Vec<Tag>, rusqlite::Error> {
        let (where_clause, mut params) = self.build_where();
        let sql = format!(
            "SELECT * FROM tags {where_clause} ORDER BY name {} LIMIT ? OFFSET ?",
            self.sort_dir.as_sql()
        );
        let offset = self.page * self.limit;
        params.push(Box::new(self.limit));
        params.push(Box::new(offset));
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let mut stmt = self.manager.conn().prepare(&sql)?;
        let rows = stmt.query_map(refs.as_slice(), Tag::from_row)?;
        rows.collect()
    }

    fn count_tags(&self) -> Result<u64, rusqlite::Error> {
        let (where_clause, params) = self.build_where();
        let sql = format!("SELECT COUNT(*) FROM tags {where_clause}");
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        self.manager
            .conn()
            .query_row(&sql, refs.as_slice(), |row| row.get(0))
    }
}

// ── query() impls ─────────────────────────────────────────────────────────────

impl<C: PostArchiverConnection> TagQuery<'_, C, NoTotal> {
    pub fn query(self) -> Result<Vec<Tag>, rusqlite::Error> {
        self.fetch_tags()
    }
}

impl<C: PostArchiverConnection> TagQuery<'_, C, WithTotal> {
    pub fn query(self) -> Result<PageResult<Tag>, rusqlite::Error> {
        let total = self.count_tags()?;
        let items = self.fetch_tags()?;
        Ok(PageResult { items, total })
    }
}

// ── Point-query helpers on PostArchiverManager ────────────────────────────────

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the tag query builder.
    pub fn tags(&self) -> TagQuery<'_, C> {
        TagQuery::new(self)
    }

    /// Fetch a single tag by primary key.
    pub fn get_tag(&self, id: TagId) -> Result<Option<Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM tags WHERE id = ?")?;
        stmt.query_row([id], Tag::from_row).optional()
    }

    /// Find a tag ID by `name` and optional `platform`.
    pub fn find_tag(
        &self,
        name: &str,
        platform: Option<PlatformId>,
    ) -> Result<Option<TagId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM tags WHERE platform IS ? AND name = ?")?;
        stmt.query_row(rusqlite::params![platform, name], |row| row.get(0))
            .optional()
    }
}
