//! Collection query builder and related point-query helpers.

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    Collection, CollectionId,
};

use super::{
    filter::{IdFilter, TextFilter},
    sortable::impl_sortable,
    BaseFilter, FromQuery, Query, Queryer, RawSql,
};

/// Fluent query builder for collections.  Obtained via [`PostArchiverManager::collections()`].
///
/// The `platform` filter accepts `Option<PlatformId>`:
/// - `Some(id)` filters collections belonging to that platform.
/// - `None` filters collections with no platform (`platform IS NULL`).
/// Multiple calls are combined with OR.
#[derive(Debug)]
pub struct CollectionQuery<'a, C> {
    queryer: Queryer<'a, C>,
    pub ids: IdFilter<CollectionId>,
    pub name: TextFilter,
    pub source: TextFilter,
}

impl<'a, C: PostArchiverConnection> CollectionQuery<'a, C> {
    pub fn new(manager: &'a PostArchiverManager<C>) -> Self {
        CollectionQuery {
            queryer: Queryer::new(manager),
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            source: TextFilter::new("source"),
        }
    }
}

impl_sortable!(CollectionQuery(CollectionSort) {
    Id: "id",
    Name: "name",
    Source: "source"
});

impl<C: PostArchiverConnection> BaseFilter for CollectionQuery<'_, C> {
    type Based = Collection;

    fn update_sql<T: FromQuery<Based = Self::Based>>(&self, mut sql: RawSql<T>) -> RawSql<T> {
        sql = self.ids.build_sql(sql);
        sql = self.name.build_sql(sql);
        sql = self.source.build_sql(sql);

        sql
    }

    fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
        &self.queryer
    }
}

impl<C: PostArchiverConnection> Query for CollectionQuery<'_, C> {
    type Wrapper<T> = Vec<T>;
    type Based = Collection;

    fn query_with_context<T: FromQuery<Based = Self::Based>>(
        self,
        sql: RawSql<T>,
    ) -> crate::error::Result<Self::Wrapper<T>> {
        let (sql, params) = sql.build_sql();
        self.queryer.fetch(&sql, params)
    }
}

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the collection query builder.
    pub fn collections(&self) -> CollectionQuery<'_, C> {
        CollectionQuery::new(self)
    }

    /// Fetch a single collection by primary key.
    pub fn get_collection(&self, id: CollectionId) -> crate::error::Result<Option<Collection>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM collections WHERE id = ?")?;
        Ok(stmt.query_row([id], Collection::from_row).optional()?)
    }

    /// Find a collection ID by `name` and optional `platform`.
    pub fn find_collection(&self, name: &str) -> crate::error::Result<Option<CollectionId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM collections WHERE platform IS ? AND name = ?")?;
        Ok(stmt
            .query_row(rusqlite::params![name], |row| row.get(0))
            .optional()?)
    }

    pub fn find_collection_by_source(
        &self,
        source: &str,
    ) -> crate::error::Result<Option<CollectionId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM collections WHERE source = ?")?;
        Ok(stmt.query_row([source], |row| row.get(0)).optional()?)
    }
}
