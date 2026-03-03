//! Tag query builder and related point-query helpers.

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    PlatformId, Tag, TagId,
};

use super::{
    filter::{IdFilter, TextFilter},
    sortable::impl_sortable,
    BaseFilter, FromQuery, Query, Queryer, RawSql,
};

/// Fluent query builder for tags.  Obtained via [`PostArchiverManager::tags()`].
///
/// The `platform` filter accepts `Option<PlatformId>`:
/// - `Some(id)` filters tags belonging to that platform.
/// - `None` filters tags with no platform (`platform IS NULL`).
/// Multiple calls are combined with OR.
#[derive(Debug)]
pub struct TagQuery<'a, C> {
    queryer: Queryer<'a, C>,
    pub ids: IdFilter<TagId>,
    pub name: TextFilter,
    pub source: TextFilter,
    pub platforms: IdFilter<PlatformId>,
}

impl<'a, C: PostArchiverConnection> TagQuery<'a, C> {
    pub fn new(manager: &'a PostArchiverManager<C>) -> Self {
        TagQuery {
            queryer: Queryer::new(manager),
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            source: TextFilter::new("source"),
            platforms: IdFilter::new("platform"),
        }
    }
}

impl_sortable!(TagQuery(TagSort) {
    Id: "id",
    Name: "name",
    Source: "source"
});

impl<C: PostArchiverConnection> BaseFilter for TagQuery<'_, C> {
    type Based = Tag;

    fn update_sql<T: FromQuery<Based = Self::Based>>(&self, mut sql: RawSql<T>) -> RawSql<T> {
        sql = self.ids.build_sql(sql);
        sql = self.name.build_sql(sql);
        sql = self.source.build_sql(sql);
        sql = self.platforms.build_sql(sql);

        sql
    }

    fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
        &self.queryer
    }
}

impl<C: PostArchiverConnection> Query for TagQuery<'_, C> {
    type Wrapper<T> = Vec<T>;
    type Based = Tag;

    fn query_with_context<T: FromQuery<Based = Self::Based>>(
        self,
        sql: RawSql<T>,
    ) -> crate::error::Result<Self::Wrapper<T>> {
        let sql = self.update_sql(sql);
        let (sql, params) = sql.build_sql();
        self.queryer.fetch(&sql, params)
    }
}

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the tag query builder.
    pub fn tags(&self) -> TagQuery<'_, C> {
        TagQuery::new(self)
    }

    /// Fetch a single tag by primary key.
    pub fn get_tag(&self, id: TagId) -> crate::error::Result<Option<Tag>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM tags WHERE id = ?")?;
        Ok(stmt.query_row([id], Tag::from_row).optional()?)
    }

    /// Find a tag ID by `name` and optional `platform`.
    pub fn find_tag(
        &self,
        name: &str,
        platform: Option<PlatformId>,
    ) -> crate::error::Result<Option<TagId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM tags WHERE platform IS ? AND name = ?")?;
        Ok(stmt
            .query_row(rusqlite::params![platform, name], |row| row.get(0))
            .optional()?)
    }
}
