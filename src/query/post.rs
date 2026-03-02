//! Post query builder and related point-query helpers.

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    AuthorId, CollectionId, PlatformId, Post, PostId, TagId,
};

use super::{
    filter::{DateFilter, IdFilter, RelationshipsFilter, TextFilter},
    sortable::impl_sortable,
    Param, Query, Queryer, RawQuery, RawSql,
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
/// # use post_archiver::query::{SortDir, Countable, Paginate, Query};
/// # use post_archiver::query::post::PostSort;
/// # let manager = PostArchiverManager::open_in_memory().unwrap();
/// let posts = manager.posts()
///     .sort(PostSort::Updated, SortDir::Desc)
///     .pagination(20, 0)
///     .query()
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
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
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

impl<C: PostArchiverConnection> RawQuery for PostQuery<'_, C> {
    type Item = Post;

    fn sql(&self) -> RawSql<Self::Item> {
        let mut sql = RawSql::new();

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
    type Wrapper<T> = Vec<T>;

    fn query_with_context(
        self,
        sql: &str,
        params: Vec<Param>,
    ) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error> {
        self.queryer().fetch(sql, params)
    }
}

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Entry point for the post query builder.
    pub fn posts(&self) -> PostQuery<'_, C> {
        PostQuery::new(self)
    }

    /// Fetch a single post by primary key. Returns `None` if not found.
    pub fn get_post(&self, id: PostId) -> Result<Option<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE id = ?")?;
        stmt.query_row([id], Post::from_row).optional()
    }

    /// Look up a post ID by its `source` field.
    pub fn find_post_by_source(&self, source: &str) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE source = ?")?;
        stmt.query_row([source], |row| row.get(0)).optional()
    }
}

mod a {
    use chrono::{DateTime, Utc};

    use crate::{
        manager::PostArchiverManager,
        query::{
            countable::Countable,
            paginate::Paginate,
            sortable::{SortDir, Sortable},
            Query, RawQuery,
        },
    };

    use super::PostSort;

    #[test]
    fn main() {
        let manager = PostArchiverManager::open_uncheck("/data/archiver")
            .unwrap()
            .unwrap();
        let mut posts = manager.posts();

        posts.updated.after(
            DateTime::parse_from_rfc3339("2024-01-01T00:00:00Z")
                .unwrap()
                .to_utc(),
        );

        let posts = posts
            .sort_random()
            .with_total()
            .sort(PostSort::Updated, SortDir::Asc);

        let posts = posts.pagination(2, 0);

        let sql = posts.sql().build_sql().0;
        let posts = posts.query().unwrap();

        panic!("posts: {posts:#?} {} {}", sql, posts.items.len());
    }
}
