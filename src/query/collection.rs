//! Collection query builder and related point-query helpers.

use std::marker::PhantomData;

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    Collection, CollectionId,
};

use super::{NoTotal, PageResult, SortDir, WithTotal};

// ── Builder ───────────────────────────────────────────────────────────────────

/// Fluent query builder for collections.  Obtained via [`PostArchiverManager::collections()`].
#[derive(Debug)]
pub struct CollectionQuery<'a, C, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    name_contains: Option<String>,
    limit: u64,
    page: u64,
    sort_dir: SortDir,
    _t: PhantomData<T>,
}

impl<'a, C, T> CollectionQuery<'a, C, T> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        CollectionQuery {
            manager,
            name_contains: None,
            limit: 50,
            page: 0,
            sort_dir: SortDir::default(),
            _t: PhantomData,
        }
    }

    /// Filter collections whose name contains `keyword` (case-insensitive `LIKE`).
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

    /// Set sort direction (collections sort by `name`).
    pub fn sort_dir(mut self, dir: SortDir) -> Self {
        self.sort_dir = dir;
        self
    }

    /// Transition: include total row count.
    pub fn with_total(self) -> CollectionQuery<'a, C, WithTotal> {
        CollectionQuery {
            manager: self.manager,
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

impl<C: PostArchiverConnection, T> CollectionQuery<'_, C, T> {
    fn build_where(&self) -> (String, Vec<BoxParam>) {
        let mut wheres: Vec<String> = Vec::new();
        let mut params: Vec<BoxParam> = Vec::new();

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

    fn fetch_collections(&self) -> Result<Vec<Collection>, rusqlite::Error> {
        let (where_clause, mut params) = self.build_where();
        let sql = format!(
            "SELECT * FROM collections {where_clause} ORDER BY name {} LIMIT ? OFFSET ?",
            self.sort_dir.as_sql()
        );
        let offset = self.page * self.limit;
        params.push(Box::new(self.limit));
        params.push(Box::new(offset));
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let mut stmt = self.manager.conn().prepare(&sql)?;
        let rows = stmt.query_map(refs.as_slice(), Collection::from_row)?;
        rows.collect()
    }

    fn count_collections(&self) -> Result<u64, rusqlite::Error> {
        let (where_clause, params) = self.build_where();
        let sql = format!("SELECT COUNT(*) FROM collections {where_clause}");
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        self.manager
            .conn()
            .query_row(&sql, refs.as_slice(), |row| row.get(0))
    }
}

// ── query() impls ─────────────────────────────────────────────────────────────

impl<C: PostArchiverConnection> CollectionQuery<'_, C, NoTotal> {
    pub fn query(self) -> Result<Vec<Collection>, rusqlite::Error> {
        self.fetch_collections()
    }
}

impl<C: PostArchiverConnection> CollectionQuery<'_, C, WithTotal> {
    pub fn query(self) -> Result<PageResult<Collection>, rusqlite::Error> {
        let total = self.count_collections()?;
        let items = self.fetch_collections()?;
        Ok(PageResult { items, total })
    }
}

// ── Point-query helpers on PostArchiverManager ────────────────────────────────

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the collection query builder.
    pub fn collections(&self) -> CollectionQuery<'_, C> {
        CollectionQuery::new(self)
    }

    /// Fetch a single collection by primary key.
    pub fn get_collection(&self, id: CollectionId) -> Result<Option<Collection>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM collections WHERE id = ?")?;
        stmt.query_row([id], Collection::from_row).optional()
    }

    /// Find a collection ID by its `source` field.
    pub fn find_collection_by_source(
        &self,
        source: &str,
    ) -> Result<Option<CollectionId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM collections WHERE source = ?")?;
        stmt.query_row([source], |row| row.get(0)).optional()
    }
}
