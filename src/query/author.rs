//! Author query builder and related point-query helpers.

use rusqlite::Connection;

use crate::{
    query::{OrderBy, Pagination, Repo, SqlBuilder},
    Author, AuthorId,
};

use super::{
    filter::{DateFilter, IdFilter, TextFilter},
    impl_sortable, Query,
};

impl<'a> Repo<'a, Author> {
    pub fn query(&self) -> AuthorQuery<'a> {
        AuthorQuery {
            conn: self.conn,
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            updated: DateFilter::new("updated"),
            order: OrderBy::default(),
            pagination: Pagination::default(),
        }
    }
}

/// Fluent query builder for authors. Obtained via `manager.authors().query()`.
///
/// # Available filter fields
/// - `ids`: filter by a set of [`AuthorId`] values.
/// - `name`: `LIKE` fuzzy match on the author name.
/// - `updated`: date-range filter on the last-updated timestamp.
#[derive(Debug)]
pub struct AuthorQuery<'a> {
    conn: &'a Connection,
    pub ids: IdFilter<AuthorId>,
    pub name: TextFilter,
    pub updated: DateFilter,

    pub order: OrderBy<AuthorSort>,
    pub pagination: Pagination,
}

impl_sortable!(AuthorSort {
    Id: "id",
    Name: "name",
    Updated: "updated"
});

impl Query for AuthorQuery<'_> {
    type Source = Author;
    fn sql(&self, sql: SqlBuilder) -> SqlBuilder {
        sql.filter(&self.ids)
            .filter(&self.name)
            .filter(&self.updated)
            .order_by(&self.order)
            .pagination(&self.pagination)
    }
    fn conn(&self) -> &Connection {
        self.conn
    }
}
