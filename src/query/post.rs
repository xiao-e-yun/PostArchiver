//! Post query builder and related point-query helpers.

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::{
    impl_sortable,
    manager::PostArchiverManager,
    query::{OrderBy, Pagination, Repo, SqlBuilder},
    AuthorId, CollectionId, PlatformId, Post, PostId, TagId,
};

use super::{
    filter::{DateFilter, IdFilter, RelationshipsFilter, TextFilter},
    FromQuery, Query,
};

impl Repo<'_, Post> {
    fn query(&self) -> PostQuery<'_> {
        PostQuery {
            conn: &self.conn,
            ids: IdFilter::new("id"),
            updated: DateFilter::new("updated"),
            published: DateFilter::new("published"),
            title: TextFilter::new("title"),
            source: TextFilter::new("source"),
            platforms: IdFilter::new("platform"),
            tags: RelationshipsFilter::new("post_tags", "tag"),
            authors: RelationshipsFilter::new("author_posts", "author"),
            collections: RelationshipsFilter::new("collection_posts", "collection"),
            order: OrderBy::default(),
            pagination: Pagination::default(),
        }
    }
}

/// Fluent query builder for posts.  Obtained via [`PostArchiverManager::posts()`].
///
/// # Available filter fields
/// - `ids`: filter by a set of [`PostId`] values.
/// - `title`: fuzzy match on the title.
/// - `source`: fuzzy match on the post source.
/// - `updated`：filter by last-updated timestamp range.
/// - `published`：filter by published timestamp range.
/// - `platforms`：filter by a set of [`PlatformId`] values.
/// - `tags`：filter by a set of [`TagId`] values, through the `post_tags` relation table.
/// - `authors`：filter by a set of [`AuthorId`] values, through the `author_posts` relation table。
/// - `collections`：filter by a set of [`CollectionId`] values, through the `collection_posts` relation table.
///
/// # Example
/// ```no_run
/// # use post_archiver::manager::PostArchiverManager;
/// # use post_archiver::query::{SortDir, Sortable, Countable, Paginate, Query};
/// # use post_archiver::query::post::PostSort;
/// # let manager = PostArchiverManager::open_in_memory().unwrap();
/// let posts = manager.posts()
///     .sort(PostSort::Updated, SortDir::Desc)
///     .pagination(20, 0)
///     .query::<post_archiver::Post>()
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct PostQuery<'a> {
    conn: &'a Connection,
    pub ids: IdFilter<PostId>,
    pub title: TextFilter,
    pub source: TextFilter,
    pub updated: DateFilter,
    pub published: DateFilter,
    pub platforms: IdFilter<PlatformId>,
    pub tags: RelationshipsFilter<TagId>,
    pub authors: RelationshipsFilter<AuthorId>,
    pub collections: RelationshipsFilter<CollectionId>,

    pub order: OrderBy<PostSort>,
    pub pagination: Pagination,
}

impl_sortable!(PostQuery(PostSort) {
    Id: "id",
    Updated: "updated",
    Published: "published",
    Title: "title"
});

impl Query for PostQuery<'_> {
    type Source = Post;
    fn sql(&self, sql: SqlBuilder) -> SqlBuilder {
        sql.filter(&self.ids)
            .filter(&self.platforms)
            .filter(&self.source)
            .filter(&self.updated)
            .filter(&self.published)
            .filter(&self.tags)
            .filter(&self.authors)
            .filter(&self.collections)
            .filter(&self.title)
            .order_by(&self.order)
            .pagination(&self.pagination)
    }
    fn conn(&self) -> &Connection {
        self.conn
    }
}
