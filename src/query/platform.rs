//! Platform query builder and related point-query helpers.

use rusqlite::Connection;

use crate::{
    query::impl_sortable,
    query::{OrderBy, Pagination, Repo, SqlBuilder},
    Platform, PlatformId,
};

use super::{
    filter::{IdFilter, TextFilter},
    Query,
};

impl<'a> Repo<'a, Platform> {
    pub fn query(&self) -> PlatformQuery<'a> {
        PlatformQuery {
            conn: self.conn,
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            order: OrderBy::default(),
            pagination: Pagination::default(),
        }
    }
}

/// Fluent query builder for platforms. Obtained via `manager.platforms().query()`.
///
/// # Available filter fields
/// - `ids`: filter by a set of [`PlatformId`] values.
/// - `name`: `LIKE` fuzzy match on the platform name.
#[derive(Debug)]
pub struct PlatformQuery<'a> {
    conn: &'a Connection,
    pub ids: IdFilter<PlatformId>,
    pub name: TextFilter,

    pub order: OrderBy<PlatformSort>,
    pub pagination: Pagination,
}

impl_sortable!(PlatformSort {
    Id: "id",
    Name: "name"
});

impl Query for PlatformQuery<'_> {
    type Source = Platform;
    fn sql(&self, sql: SqlBuilder) -> SqlBuilder {
        sql.filter(&self.ids)
            .filter(&self.name)
            .order_by(&self.order)
            .pagination(&self.pagination)
    }
    fn conn(&self) -> &Connection {
        self.conn
    }
}
