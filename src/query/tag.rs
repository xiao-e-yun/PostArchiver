//! Tag query builder and related point-query helpers.

use rusqlite::Connection;

use crate::{
    impl_sortable,
    query::{author::AuthorSort, OrderBy, Pagination, Repo, SqlBuilder},
    PlatformId, Tag, TagId,
};

use super::{
    filter::{IdFilter, TextFilter},
    Query,
};

impl Repo<'_, Tag> {
    fn query(&self) -> TagQuery<'_> {
        TagQuery {
            conn: &self.conn,
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            platforms: IdFilter::new("platform"),
            order: OrderBy::default(),
            pagination: Pagination::default(),
        }
    }
}

/// Fluent query builder for tags.  Obtained via [`PostArchiverManager::tags()`].
///
/// # Available filter fields
/// - `ids`: filter by a set of [`TagId`] values.
/// - `name`: `LIKE` fuzzy match on the tag name.
/// - `source`: `LIKE` fuzzy match on the source field.
/// - `platforms`: filter by a set of [`PlatformId`] values.
#[derive(Debug)]
pub struct TagQuery<'a> {
    conn: &'a Connection,
    pub ids: IdFilter<TagId>,
    pub platforms: IdFilter<PlatformId>,
    pub name: TextFilter,

    pub order: OrderBy<AuthorSort>,
    pub pagination: Pagination,
}

impl_sortable!(TagQuery(TagSort) {
    Id: "id",
    Name: "name",
    Platform: "platform"
});

impl Query for TagQuery<'_> {
    type Source = Tag;
    fn sql(&self, sql: SqlBuilder) -> SqlBuilder {
        sql.filter(&self.ids)
            .filter(&self.platforms)
            .filter(&self.name)
            .order_by(&self.order)
            .pagination(&self.pagination)
    }
    fn conn(&self) -> &Connection {
        self.conn
    }
}
