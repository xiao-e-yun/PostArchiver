//! Author query builder and related point-query helpers.

use rusqlite::Connection;

use crate::{
    query::{OrderBy, Pagination, Repo, SqlBuilder},
    Alias, Author, AuthorId,
};

use super::{
    filter::{DateFilter, IdFilter, TextFilter},
    impl_sortable, FromQuery, Query,
};

impl Repo<'_, Author> {
    fn query(&self) -> AuthorQuery<'_> {
        AuthorQuery {
            conn: &self.conn,
            ids: IdFilter::new("id"),
            name: TextFilter::new("name"),
            updated: DateFilter::new("updated"),
            order: OrderBy::default(),
            pagination: Pagination::default(),
        }
    }
    //
    pub fn list_aliases(&self, id: AuthorId) -> crate::error::Result<Vec<Alias>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT * FROM author_aliases WHERE target = ?")?;
        let rows = stmt.query_map([id], Alias::from_row)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }
}

/// Fluent query builder for authors.  Obtained via [`PostArchiverManager::authors()`].
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

impl_sortable!(AuthorQuery(AuthorSort) {
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
