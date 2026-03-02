//! Tag query builder and related point-query helpers.

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    PlatformId, Post, Tag, TagId,
};

use super::{
    filter::{IdFilter, TextFilter},
    sortable::impl_sortable,
    Query, Queryer, RawQuery, RawSql,
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
    ids: IdFilter<TagId>,
    name: TextFilter,
    source: TextFilter,
    platforms: IdFilter<PlatformId>,
}

impl<'a, C: PostArchiverConnection> TagQuery<'a, C> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
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

impl<C: PostArchiverConnection> RawQuery for TagQuery<'_, C> {
    type Item = Tag;

    fn sql(&self) -> RawSql<Self::Item> {
        let mut sql = RawSql::new();

        sql = self.ids.build_sql(sql);
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

    fn query_with_context(
        self,
        sql: &str,
        params: Vec<super::Param>,
    ) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error> {
        self.queryer().fetch(&sql, params)
    }
}

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
