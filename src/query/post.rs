//! Post query builder and related point-query helpers.

use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, CollectionId, PlatformId, Post, PostId, TagId,
};

use super::{
    filter::{DateFilter, IdFilter, RelationshipsFilter, TextFilter},
    sortable::impl_sortable,
    BaseFilter, FromQuery, Query, Queryer, RawSql,
};

impl_sortable!(PostQuery(PostSort) {
    Id: "id",
    Updated: "updated",
    Published: "published",
    Title: "title"
});

// ── Builder ───────────────────────────────────────────────────────────────────

/// Fluent query builder for posts.
///
/// Obtained via [`PostArchiverManager::posts()`].
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
pub struct PostQuery<'a, C> {
    queryer: Queryer<'a, C>,
    pub ids: IdFilter<PostId>,
    pub title: TextFilter,
    pub source: TextFilter,
    pub updated: DateFilter,
    pub published: DateFilter,
    pub platforms: IdFilter<PlatformId>,
    pub tags: RelationshipsFilter<TagId>,
    pub authors: RelationshipsFilter<AuthorId>,
    pub collections: RelationshipsFilter<CollectionId>,
}

// ── Builder methods ───────────────────────────────────────────────────────────

impl<'a, C: PostArchiverConnection> PostQuery<'a, C> {
    pub fn new(manager: &'a PostArchiverManager<C>) -> Self {
        PostQuery {
            queryer: Queryer::new(manager),
            ids: IdFilter::new("id"),
            title: TextFilter::new("title"),
            source: TextFilter::new("source"),
            updated: DateFilter::new("updated"),
            published: DateFilter::new("published"),
            platforms: IdFilter::new("platform"),
            tags: RelationshipsFilter::new("post_tags", "tag"),
            authors: RelationshipsFilter::new("author_posts", "author"),
            collections: RelationshipsFilter::new("collection_posts", "collection"),
        }
    }
}

// ── Trait impls ───────────────────────────────────────────────────────────────

impl<C: PostArchiverConnection> BaseFilter for PostQuery<'_, C> {
    type Based = Post;

    fn update_sql<T: FromQuery<Based = Self::Based>>(&self, mut sql: RawSql<T>) -> RawSql<T> {
        sql = self.ids.build_sql(sql);
        sql = self.title.build_sql(sql);
        sql = self.source.build_sql(sql);
        sql = self.updated.build_sql(sql);
        sql = self.published.build_sql(sql);
        sql = self.platforms.build_sql(sql);
        sql = self.authors.build_sql(sql);
        sql = self.tags.build_sql(sql);
        sql = self.collections.build_sql(sql);

        sql
    }

    fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
        &self.queryer
    }
}

impl<C: PostArchiverConnection> Query for PostQuery<'_, C> {
    type Wrapper<U> = Vec<U>;
    type Based = Post;

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
    /// Entry point for the post query builder.
    pub fn posts(&self) -> PostQuery<'_, C> {
        PostQuery::new(self)
    }

    /// Fetch a single post by primary key. Returns `None` if not found.
    pub fn get_post(&self, id: PostId) -> crate::error::Result<Option<Post>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE id = ?")?;
        Ok(stmt.query_row([id], Post::from_row).optional()?)
    }

    /// Look up a post ID by its `source` field.
    pub fn find_post(&self, source: &str) -> crate::error::Result<Option<PostId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE source = ?")?;
        Ok(stmt.query_row([source], |row| row.get(0)).optional()?)
    }

    /// Look up a post ID by its `source` field, but only if its `updated` timestamp is newer than
    /// the given value.
    pub fn find_post_with_updated(
        &self,
        source: &str,
        updated: &DateTime<Utc>,
    ) -> crate::error::Result<Option<PostId>> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE source = ? AND updated >= ?")?;
        Ok(stmt
            .query_row(params![source, updated], |row| row.get(0))
            .optional()?)
    }
}
