//! FileMeta point-query helpers (no builder — use the direct methods below).

use rusqlite::Connection;

use crate::{
    query::impl_sortable,
    query::{
        filter::{IdFilter, TextFilter},
        OrderBy, Pagination, Query, Repo, SqlBuilder,
    },
    FileMeta, FileMetaId, PostId,
};

impl<'a> Repo<'a, FileMeta> {
    pub fn query(&self) -> FileMetaQuery<'a> {
        FileMetaQuery {
            conn: self.conn,
            ids: IdFilter::new("id"),
            posts: IdFilter::new("post"),
            filename: TextFilter::new("filename"),
            mime: TextFilter::new("mime"),
            order: OrderBy::default(),
            pagination: Pagination::default(),
        }
    }
}

/// Fluent query builder for file metadata. Obtained via `manager.file_metas().query()`.
///
/// # Available filter fields
/// - `ids`: filter by a set of [`FileMetaId`] values.
/// - `posts`: filter by the owning [`PostId`]s.
/// - `filename`: `LIKE` fuzzy match on the filename.
/// - `mime`: `LIKE` fuzzy match on the MIME type.
#[derive(Debug)]
pub struct FileMetaQuery<'a> {
    conn: &'a Connection,
    pub ids: IdFilter<FileMetaId>,
    pub posts: IdFilter<PostId>,
    pub filename: TextFilter,
    pub mime: TextFilter,

    pub order: OrderBy<FileMetaSort>,
    pub pagination: Pagination,
}

impl_sortable!(FileMetaSort {
    Id: "id",
    Post: "post",
    Filename: "filename",
    Mime: "mime"
});

impl Query for FileMetaQuery<'_> {
    type Source = FileMeta;
    fn sql(&self, sql: SqlBuilder) -> SqlBuilder {
        sql.filter(&self.ids)
            .filter(&self.posts)
            .filter(&self.filename)
            .filter(&self.mime)
            .order_by(&self.order)
            .pagination(&self.pagination)
    }
    fn conn(&self) -> &Connection {
        self.conn
    }
}
