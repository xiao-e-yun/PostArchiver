//! Author query builder and related point-query helpers.

use std::marker::PhantomData;

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    Alias, Author, AuthorId, PlatformId, Post,
};

use super::{NoRelations, NoTotal, PageResult, SortDir, WithRelations, WithTotal};

// ── Supporting types ──────────────────────────────────────────────────────────

/// Sort field for [`AuthorQuery`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AuthorSort {
    #[default]
    Updated,
    Name,
    Id,
}

impl AuthorSort {
    fn as_col(self) -> &'static str {
        match self {
            AuthorSort::Updated => "updated",
            AuthorSort::Name => "name",
            AuthorSort::Id => "id",
        }
    }
}

/// An [`Author`] with all aliases and posts eagerly loaded.
#[derive(Debug, Clone)]
pub struct AuthorWithRelations {
    pub author: Author,
    pub aliases: Vec<Alias>,
    pub posts: Vec<Post>,
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Fluent query builder for authors.  Obtained via [`PostArchiverManager::authors()`].
#[derive(Debug)]
pub struct AuthorQuery<'a, C, R = NoRelations, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    name_contains: Option<String>,
    limit: u64,
    page: u64,
    sort: AuthorSort,
    sort_dir: SortDir,
    _r: PhantomData<R>,
    _t: PhantomData<T>,
}

impl<'a, C, R, T> AuthorQuery<'a, C, R, T> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        AuthorQuery {
            manager,
            name_contains: None,
            limit: 50,
            page: 0,
            sort: AuthorSort::default(),
            sort_dir: SortDir::default(),
            _r: PhantomData,
            _t: PhantomData,
        }
    }

    /// Filter authors whose name contains `keyword` (case-insensitive `LIKE`).
    pub fn name_contains(mut self, keyword: impl Into<String>) -> Self {
        self.name_contains = Some(keyword.into());
        self
    }

    /// Set pagination (`limit` rows per page, 0-based `page`).
    pub fn pagination(mut self, limit: u64, page: u64) -> Self {
        self.limit = limit;
        self.page = page;
        self
    }

    /// Set sort field and direction.
    pub fn sort(mut self, sort: AuthorSort, dir: SortDir) -> Self {
        self.sort = sort;
        self.sort_dir = dir;
        self
    }

    /// Transition: load aliases and posts alongside each author.
    pub fn relations(self) -> AuthorQuery<'a, C, WithRelations, T> {
        AuthorQuery {
            manager: self.manager,
            name_contains: self.name_contains,
            limit: self.limit,
            page: self.page,
            sort: self.sort,
            sort_dir: self.sort_dir,
            _r: PhantomData,
            _t: self._t,
        }
    }

    /// Transition: include total row count in the result.
    pub fn with_total(self) -> AuthorQuery<'a, C, R, WithTotal> {
        AuthorQuery {
            manager: self.manager,
            name_contains: self.name_contains,
            limit: self.limit,
            page: self.page,
            sort: self.sort,
            sort_dir: self.sort_dir,
            _r: self._r,
            _t: PhantomData,
        }
    }
}

// ── Internal SQL helpers ──────────────────────────────────────────────────────

type BoxParam = Box<dyn rusqlite::types::ToSql>;

impl<C: PostArchiverConnection, R, T> AuthorQuery<'_, C, R, T> {
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

    fn order_clause(&self) -> String {
        format!("ORDER BY {} {}", self.sort.as_col(), self.sort_dir.as_sql())
    }

    fn fetch_authors(&self) -> Result<Vec<Author>, rusqlite::Error> {
        let (where_clause, mut params) = self.build_where();
        let order = self.order_clause();
        let sql = format!("SELECT * FROM authors {where_clause} {order} LIMIT ? OFFSET ?");
        let offset = self.page * self.limit;
        params.push(Box::new(self.limit));
        params.push(Box::new(offset));
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let mut stmt = self.manager.conn().prepare(&sql)?;
        let rows = stmt.query_map(refs.as_slice(), Author::from_row)?;
        rows.collect()
    }

    fn count_authors(&self) -> Result<u64, rusqlite::Error> {
        let (where_clause, params) = self.build_where();
        let sql = format!("SELECT COUNT(*) FROM authors {where_clause}");
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        self.manager
            .conn()
            .query_row(&sql, refs.as_slice(), |row| row.get(0))
    }
}

// ── query() impls ─────────────────────────────────────────────────────────────

impl<C: PostArchiverConnection> AuthorQuery<'_, C, NoRelations, NoTotal> {
    pub fn query(self) -> Result<Vec<Author>, rusqlite::Error> {
        self.fetch_authors()
    }
}

impl<C: PostArchiverConnection> AuthorQuery<'_, C, NoRelations, WithTotal> {
    pub fn query(self) -> Result<PageResult<Author>, rusqlite::Error> {
        let total = self.count_authors()?;
        let items = self.fetch_authors()?;
        Ok(PageResult { items, total })
    }
}

impl<C: PostArchiverConnection> AuthorQuery<'_, C, WithRelations, NoTotal> {
    pub fn query(self) -> Result<Vec<AuthorWithRelations>, rusqlite::Error> {
        let authors = self.fetch_authors()?;
        authors
            .into_iter()
            .map(|author| {
                let id = author.id;
                Ok(AuthorWithRelations {
                    aliases: self.manager.list_author_aliases(id)?,
                    posts: self.manager.list_author_posts(id)?,
                    author,
                })
            })
            .collect()
    }
}

impl<C: PostArchiverConnection> AuthorQuery<'_, C, WithRelations, WithTotal> {
    pub fn query(self) -> Result<PageResult<AuthorWithRelations>, rusqlite::Error> {
        let total = self.count_authors()?;
        let authors = self.fetch_authors()?;
        let items = authors
            .into_iter()
            .map(|author| {
                let id = author.id;
                Ok(AuthorWithRelations {
                    aliases: self.manager.list_author_aliases(id)?,
                    posts: self.manager.list_author_posts(id)?,
                    author,
                })
            })
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(PageResult { items, total })
    }
}

// ── Point-query helpers on PostArchiverManager ────────────────────────────────

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the author query builder.
    pub fn authors(&self) -> AuthorQuery<'_, C> {
        AuthorQuery::new(self)
    }

    /// Fetch a single author by primary key.
    pub fn get_author(&self, id: AuthorId) -> Result<Option<Author>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM authors WHERE id = ?")?;
        stmt.query_row([id], Author::from_row).optional()
    }

    /// Find an author ID by their alias (`source` + `platform`).
    pub fn find_author_by_alias(
        &self,
        source: &str,
        platform: PlatformId,
    ) -> Result<Option<AuthorId>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT target FROM author_aliases WHERE platform = ? AND source = ?",
        )?;
        stmt.query_row(rusqlite::params![platform, source], |row| row.get(0))
            .optional()
    }

    /// Fetch all aliases of an author.
    pub(crate) fn list_author_aliases(
        &self,
        author: AuthorId,
    ) -> Result<Vec<Alias>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let rows = stmt.query_map([author], Alias::from_row)?;
        rows.collect()
    }

    /// Fetch all posts by an author (full entities).
    pub(crate) fn list_author_posts(&self, author: AuthorId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT posts.* FROM posts \
             INNER JOIN author_posts ON author_posts.post = posts.id \
             WHERE author_posts.author = ?",
        )?;
        let rows = stmt.query_map([author], Post::from_row)?;
        rows.collect()
    }
}
