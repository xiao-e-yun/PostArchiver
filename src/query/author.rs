//! Author query builder and related point-query helpers.

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    Alias, Author, AuthorId, PlatformId,
};

use super::{
    filter::{DateFilter, IdFilter, TextFilter},
    sortable::impl_sortable,
    BaseQuery, Query, Queryer, RawSql,
};

/// Fluent query builder for authors.  Obtained via [`PostArchiverManager::authors()`].
///
/// The `platform` filter accepts `Option<PlatformId>`:
/// - `Some(id)` filters authors belonging to that platform.
/// - `None` filters authors with no platform (`platform IS NULL`).
/// Multiple calls are combined with OR.
#[derive(Debug)]
pub struct AuthorQuery<'a, C> {
    queryer: Queryer<'a, C>,
    ids: IdFilter<AuthorId>,
    name: TextFilter,
    updated: DateFilter,
}

impl<'a, C: PostArchiverConnection> AuthorQuery<'a, C> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        AuthorQuery {
            queryer: Queryer::new(manager),
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            updated: DateFilter::new("updated"),
        }
    }
}

impl_sortable!(AuthorQuery(AuthorSort) {
    Id: "id",
    Name: "name",
    Updated: "updated"
});

impl<C: PostArchiverConnection> BaseQuery for AuthorQuery<'_, C> {
    type Item = Author;

    fn sql(&self) -> RawSql<Self::Item> {
        let mut sql = RawSql::new();

        sql = self.ids.build_sql(sql);
        sql = self.name.build_sql(sql);
        sql = self.updated.build_sql(sql);

        sql
    }

    fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
        &self.queryer
    }
}

impl<C: PostArchiverConnection> Query for AuthorQuery<'_, C> {
    type Wrapper<T> = Vec<T>;

    fn query_with_context(
        self,
        sql: &str,
        params: Vec<super::Param>,
    ) -> crate::error::Result<Self::Wrapper<Self::Item>> {
        self.queryer().fetch(sql, params)
    }
}

// ── Point-query helpers on PostArchiverManager ────────────────────────────────

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the author query builder.
    pub fn authors(&self) -> AuthorQuery<'_, C> {
        AuthorQuery::new(self)
    }

    /// Fetch a single author by primary key.
    pub fn get_author(&self, id: AuthorId) -> crate::error::Result<Option<Author>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM authors WHERE id = ?")?;
        Ok(stmt.query_row([id], Author::from_row).optional()?)
    }

    /// Find an author ID by their alias (`source` + `platform`).
    pub fn find_author_by_alias(
        &self,
        source: &str,
        platform: PlatformId,
    ) -> crate::error::Result<Option<AuthorId>> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT target FROM author_aliases WHERE platform = ? AND source = ?",
        )?;
        Ok(stmt
            .query_row(rusqlite::params![platform, source], |row| row.get(0))
            .optional()?)
    }

    /// Fetch all aliases of an author.
    pub fn list_author_aliases(&self, author: AuthorId) -> crate::error::Result<Vec<Alias>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let rows = stmt.query_map([author], Alias::from_row)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }
}
