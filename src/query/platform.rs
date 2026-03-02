//! Platform query builder and related point-query helpers.

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    Platform, PlatformId,
};

use super::{
    filter::{IdFilter, TextFilter},
    sortable::impl_sortable,
    BaseQuery, Query, Queryer, RawSql,
};

// ── Builder ───────────────────────────────────────────────────────────────────

/// Query builder for platforms.  Obtained via [`PostArchiverManager::platforms()`].
///
/// Platforms are few, so this builder is intentionally simple — no pagination or
/// count is provided.  `.query()` always returns `Vec<Platform>`.
#[derive(Debug)]
pub struct PlatformQuery<'a, C> {
    queryer: Queryer<'a, C>,
    pub ids: IdFilter<PlatformId>,
    pub name: TextFilter,
}

impl_sortable!(PlatformQuery(PlatformSort) {
    Id: "id",
    Name: "name"
});

impl<'a, C: PostArchiverConnection> PlatformQuery<'a, C> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        PlatformQuery {
            queryer: Queryer::new(manager),
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
        }
    }
}

impl<C: PostArchiverConnection> BaseQuery for PlatformQuery<'_, C> {
    type Item = Platform;

    fn sql(&self) -> RawSql<Self::Item> {
        let mut sql = RawSql::new();

        sql = self.ids.build_sql(sql);
        sql = self.name.build_sql(sql);

        sql
    }

    fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
        &self.queryer
    }
}

impl<C: PostArchiverConnection> Query for PlatformQuery<'_, C> {
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
    /// Entry point for the platform query builder.
    pub fn platforms(&self) -> PlatformQuery<'_, C> {
        PlatformQuery::new(self)
    }

    /// Fetch a single platform by primary key.
    pub fn get_platform(&self, id: PlatformId) -> crate::error::Result<Option<Platform>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM platforms WHERE id = ?")?;
        Ok(stmt.query_row([id], Platform::from_row).optional()?)
    }

    /// Find a platform ID by name (exact match, case-sensitive).
    pub fn find_platform(&self, name: &str) -> crate::error::Result<Option<PlatformId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM platforms WHERE name = ?")?;
        Ok(stmt.query_row([name], |row| row.get(0)).optional()?)
    }
}
