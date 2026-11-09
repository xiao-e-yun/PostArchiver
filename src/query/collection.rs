//! Collection query builder and related point-query helpers.

use rusqlite::Connection;

use crate::{
    impl_sortable,
    query::{OrderBy, Pagination, Query, Repo, SqlBuilder},
    Collection, CollectionId,
};

use super::filter::{IdFilter, TextFilter};

impl Repo<'_, Collection> {
    pub fn query(&self) -> CollectionQuery<'_> {
        CollectionQuery {
            conn: self.conn,
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            source: TextFilter::new("source"),
            order: OrderBy::default(),
            pagination: Pagination::default(),
        }
    }
}

/// Fluent query builder for collections.  Obtained via [`PostArchiverManager::collections()`].
///
/// # Available filter fields
/// - `ids`: filter by a set of [`CollectionId`] values.
/// - `name`: `LIKE` fuzzy match on the collection name.
/// - `source`: `LIKE` fuzzy match on the source field.
#[derive(Debug)]
pub struct CollectionQuery<'a> {
    conn: &'a Connection,
    pub ids: IdFilter<CollectionId>,
    pub name: TextFilter,
    pub source: TextFilter,

    pub order: OrderBy<CollectionSort>,
    pub pagination: Pagination,
}

impl_sortable!(CollectionQuery(CollectionSort) {
    Id: "id",
    Name: "name",
    Source: "source"
});

impl Query for CollectionQuery<'_> {
    type Source = Collection;
    fn sql(&self, sql: SqlBuilder) -> SqlBuilder {
        sql.filter(&self.ids)
            .filter(&self.name)
            .filter(&self.source)
            .order_by(&self.order)
            .pagination(&self.pagination)
    }
    fn conn(&self) -> &Connection {
        self.conn
    }
}
