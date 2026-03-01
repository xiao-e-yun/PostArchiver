//! Post query builder and related point-query helpers.

use std::marker::PhantomData;

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    Author, AuthorId, Collection, CollectionId, FileMeta, PlatformId, Post, PostId, Tag, TagId,
};

use super::{NoRelations, NoTotal, PageResult, SortDir, WithRelations, WithTotal};

// ── Supporting types ──────────────────────────────────────────────────────────

/// Sort field for [`PostQuery`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PostSort {
    #[default]
    Updated,
    Published,
    Title,
    Id,
}

impl PostSort {
    fn as_col(self) -> &'static str {
        match self {
            PostSort::Updated => "updated",
            PostSort::Published => "published",
            PostSort::Title => "title",
            PostSort::Id => "id",
        }
    }
}

/// A [`Post`] with all of its relations eagerly loaded.
#[derive(Debug, Clone)]
pub struct PostWithRelations {
    pub post: Post,
    pub authors: Vec<Author>,
    pub tags: Vec<Tag>,
    pub file_metas: Vec<FileMeta>,
    pub collections: Vec<Collection>,
}

// ── Builder ───────────────────────────────────────────────────────────────────

/// Fluent query builder for posts.
///
/// Obtained via [`PostArchiverManager::posts()`].
///
/// The type parameters `R` and `T` are [typestate] markers:
/// - `R ∈ {NoRelations, WithRelations}` — controls whether `.query()` loads relations
/// - `T ∈ {NoTotal, WithTotal}` — controls whether `.query()` wraps results in [`PageResult`]
///
/// [typestate]: https://docs.rust-embedded.org/book/static-guarantees/typestate-programming.html
///
/// # Example
/// ```no_run
/// # use post_archiver::manager::PostArchiverManager;
/// # use post_archiver::query::{SortDir};
/// # use post_archiver::query::post::PostSort;
/// # let manager = PostArchiverManager::open_in_memory().unwrap();
/// let posts = manager.posts()
///     .pagination(20, 0)
///     .sort(PostSort::Updated, SortDir::Desc)
///     .query()
///     .unwrap();
/// ```
#[derive(Debug)]
pub struct PostQuery<'a, C, R = NoRelations, T = NoTotal> {
    manager: &'a PostArchiverManager<C>,
    platforms: Vec<PlatformId>,
    tags: Vec<TagId>,
    collections: Vec<CollectionId>,
    authors: Vec<AuthorId>,
    ids: Vec<PostId>,
    limit: u64,
    page: u64,
    sort: PostSort,
    sort_dir: SortDir,
    _r: PhantomData<R>,
    _t: PhantomData<T>,
}

// ── Builder methods (generic over R and T) ────────────────────────────────────

impl<'a, C, R, T> PostQuery<'a, C, R, T> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        PostQuery {
            manager,
            platforms: Vec::new(),
            tags: Vec::new(),
            collections: Vec::new(),
            authors: Vec::new(),
            ids: Vec::new(),
            limit: 50,
            page: 0,
            sort: PostSort::default(),
            sort_dir: SortDir::default(),
            _r: PhantomData,
            _t: PhantomData,
        }
    }

    /// Filter posts by a single platform (additive, OR semantics with other `platform` calls).
    pub fn platform(mut self, id: PlatformId) -> Self {
        self.platforms.push(id);
        self
    }

    /// Filter posts by multiple platforms (OR semantics).
    pub fn platforms(mut self, ids: impl IntoIterator<Item = PlatformId>) -> Self {
        self.platforms.extend(ids);
        self
    }

    /// Filter posts belonging to a specific collection.
    pub fn collection(mut self, id: CollectionId) -> Self {
        self.collections.push(id);
        self
    }

    pub fn collections(mut self, ids: impl IntoIterator<Item = CollectionId>) -> Self {
        self.collections.extend(ids);
        self
    }

    pub fn tag(mut self, id: TagId) -> Self {
        self.tags.push(id);
        self
    }

    /// Filter posts that belong to **all** of the given tags (AND semantics).
    pub fn tags(mut self, ids: impl IntoIterator<Item = TagId>) -> Self {
        self.tags.extend(ids);
        self
    }

    /// Filter posts by author.
    pub fn author(mut self, id: AuthorId) -> Self {
        self.authors.push(id);
        self
    }

    pub fn authors(mut self, ids: impl IntoIterator<Item = AuthorId>) -> Self {
        self.authors.extend(ids);
        self
    }

    /// Filter to a single post id (additive, IN semantics with other `id` calls).
    pub fn id(mut self, id: PostId) -> Self {
        self.ids.push(id);
        self
    }

    /// Filter to multiple post ids (IN semantics).
    pub fn ids(mut self, ids: impl IntoIterator<Item = PostId>) -> Self {
        self.ids.extend(ids);
        self
    }

    /// Set pagination (`limit` rows per page, 0-based `page` number).
    pub fn pagination(mut self, limit: u64, page: u64) -> Self {
        self.limit = limit;
        self.page = page;
        self
    }

    /// Set sort field and direction.
    pub fn sort(mut self, sort: PostSort, dir: SortDir) -> Self {
        self.sort = sort;
        self.sort_dir = dir;
        self
    }

    /// Transition: load all relations (authors, tags, files, collections) alongside each post.
    pub fn relations(self) -> PostQuery<'a, C, WithRelations, T> {
        PostQuery {
            manager: self.manager,
            platforms: self.platforms,
            tags: self.tags,
            collections: self.collections,
            authors: self.authors,
            ids: self.ids,
            limit: self.limit,
            page: self.page,
            sort: self.sort,
            sort_dir: self.sort_dir,
            _r: PhantomData,
            _t: self._t,
        }
    }

    /// Transition: include total row count in the result ([`PageResult`]).
    pub fn with_total(self) -> PostQuery<'a, C, R, WithTotal> {
        PostQuery {
            manager: self.manager,
            platforms: self.platforms,
            tags: self.tags,
            collections: self.collections,
            authors: self.authors,
            ids: self.ids,
            limit: self.limit,
            page: self.page,
            sort: self.sort,
            sort_dir: self.sort_dir,
            _r: self._r,
            _t: PhantomData,
        }
    }
}

// ── Internal SQL helpers ──────────────────────────────────────────────────────

type BoxParam = Box<dyn rusqlite::types::ToSql>;

impl<C: PostArchiverConnection, R, T> PostQuery<'_, C, R, T> {
    /// Builds `(WHERE …, params)` from the current filter state.
    fn build_where(&self) -> (String, Vec<BoxParam>) {
        let mut wheres: Vec<String> = Vec::new();
        let mut params: Vec<BoxParam> = Vec::new();

        match self.ids.len() {
            0 => {}
            1 => {
                wheres.push("id = ?".to_string());
                params.push(Box::new(self.ids[0]));
            }
            _ => {
                wheres.push("id IN (SELECT value FROM json_each(?))".to_string());
                let json_array = serde_json::to_string(&self.ids).unwrap();
                params.push(Box::new(json_array));
            }
        }

        match self.platforms.len() {
            0 => {}
            1 => {
                wheres.push("platform = ?".to_string());
                params.push(Box::new(self.platforms[0]));
            }
            _ => {
                wheres.push("platform IN (SELECT value FROM json_each(?))".to_string());
                let json_array = serde_json::to_string(&self.platforms).unwrap();
                params.push(Box::new(json_array));
            }
        }

        macro_rules! add_relation_filter {
            ($ty:ident $field:ident, $table:ident, $col:ident) => {
                match self.$field.len() {
                    0 => {}
                    1 => {
                        wheres.push(format!(
                                "EXISTS (SELECT 1 FROM {} WHERE post = posts.id AND {} = ?)",
                                stringify!($table), stringify!($col)
                        ));
                        params.push(Box::new(self.$field[0]));
                    }
                    #[allow(unused_variables)]
                    n => {
                        add_relation_filter!($ty(n) => $field, $table, $col);
                    }
                }
            };
            (AND($count:expr) => $field:ident, $table:ident, $col:ident) => {
                wheres.push(format!(
                    "? == (SELECT COUNT(*) FROM {} WHERE post = posts.id AND {} IN (SELECT value FROM json_each(?)))",
                    stringify!($table), stringify!($col)
                ));
                params.push(Box::new(self.$field.len() as u64));
                let json_array = serde_json::to_string(&self.$field).unwrap();
                params.push(Box::new(json_array));
            };
            (OR($count: expr) => $field:ident, $table:ident, $col:ident) => {
                wheres.push(format!(
                    "EXISTS (SELECT 1 FROM {} WHERE post = posts.id AND {} IN (SELECT value FROM json_each(?)))",
                    stringify!($table), stringify!($col)
                ));

                let json_array = serde_json::to_string(&self.$field).unwrap();
                params.push(Box::new(json_array));
            };
        }

        add_relation_filter!(AND authors, author_posts, author);
        add_relation_filter!(AND tags, post_tags, tag);
        add_relation_filter!(AND collections, collection_posts, collection);

        let clause = if wheres.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", wheres.join(" AND "))
        };

        (clause, params)
    }

    fn order_clause(&self) -> String {
        format!("ORDER BY {} {}", self.sort.as_col(), self.sort_dir.as_sql())
    }

    /// Execute into a plain `Vec<Post>`.
    fn fetch_posts(&self) -> Result<Vec<Post>, rusqlite::Error> {
        let (where_clause, mut params) = self.build_where();
        let order = self.order_clause();
        let sql = format!("SELECT * FROM posts {where_clause} {order} LIMIT ? OFFSET ?");
        let offset = self.page * self.limit;
        params.push(Box::new(self.limit));
        params.push(Box::new(offset));
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        let mut stmt = self.manager.conn().prepare(&sql)?;
        let rows = stmt.query_map(refs.as_slice(), Post::from_row)?;
        rows.collect()
    }

    /// Count total matching rows (ignores pagination).
    fn count_posts(&self) -> Result<u64, rusqlite::Error> {
        let (where_clause, params) = self.build_where();
        let sql = format!("SELECT COUNT(*) FROM posts {where_clause}");
        let refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|b| b.as_ref()).collect();
        self.manager
            .conn()
            .query_row(&sql, refs.as_slice(), |row| row.get(0))
    }
}

// ── query() impls — 4 typestate combinations ──────────────────────────────────

impl<C: PostArchiverConnection> PostQuery<'_, C, NoRelations, NoTotal> {
    /// Execute and return matching posts.
    pub fn query(self) -> Result<Vec<Post>, rusqlite::Error> {
        self.fetch_posts()
    }
}

impl<C: PostArchiverConnection> PostQuery<'_, C, NoRelations, WithTotal> {
    /// Execute and return matching posts with total count.
    pub fn query(self) -> Result<PageResult<Post>, rusqlite::Error> {
        let total = self.count_posts()?;
        let items = self.fetch_posts()?;
        Ok(PageResult { items, total })
    }
}

impl<C: PostArchiverConnection> PostQuery<'_, C, WithRelations, NoTotal> {
    /// Execute and return posts with all relations loaded.
    pub fn query(self) -> Result<Vec<PostWithRelations>, rusqlite::Error> {
        let posts = self.fetch_posts()?;
        posts
            .into_iter()
            .map(|post| {
                let id = post.id;
                Ok(PostWithRelations {
                    post,
                    authors: self.manager.list_post_authors(id)?,
                    tags: self.manager.list_post_tags(id)?,
                    file_metas: self.manager.list_post_files(id)?,
                    collections: self.manager.list_post_collections(id)?,
                })
            })
            .collect()
    }
}

impl<C: PostArchiverConnection> PostQuery<'_, C, WithRelations, WithTotal> {
    /// Execute and return posts with all relations loaded, plus total count.
    pub fn query(self) -> Result<PageResult<PostWithRelations>, rusqlite::Error> {
        let total = self.count_posts()?;
        let posts = self.fetch_posts()?;
        let items = posts
            .into_iter()
            .map(|post| {
                let id = post.id;
                Ok(PostWithRelations {
                    post,
                    authors: self.manager.list_post_authors(id)?,
                    tags: self.manager.list_post_tags(id)?,
                    file_metas: self.manager.list_post_files(id)?,
                    collections: self.manager.list_post_collections(id)?,
                })
            })
            .collect::<Result<Vec<_>, rusqlite::Error>>()?;
        Ok(PageResult { items, total })
    }
}

// ── Point-query helpers on PostArchiverManager ────────────────────────────────

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

    /// Fetch a post together with all of its relations in one call.
    pub fn get_post_with_relations(
        &self,
        id: PostId,
    ) -> Result<Option<PostWithRelations>, rusqlite::Error> {
        let Some(post) = self.get_post(id)? else {
            return Ok(None);
        };
        Ok(Some(PostWithRelations {
            authors: self.list_post_authors(id)?,
            tags: self.list_post_tags(id)?,
            file_metas: self.list_post_files(id)?,
            collections: self.list_post_collections(id)?,
            post,
        }))
    }

    /// Fetch all authors of a post (full entities).
    pub(crate) fn list_post_authors(&self, post: PostId) -> Result<Vec<Author>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT authors.* FROM authors \
             INNER JOIN author_posts ON author_posts.author = authors.id \
             WHERE author_posts.post = ?",
        )?;
        let rows = stmt.query_map([post], Author::from_row)?;
        rows.collect()
    }

    /// Fetch all tags of a post (full entities).
    pub(crate) fn list_post_tags(&self, post: PostId) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT tags.* FROM tags \
             INNER JOIN post_tags ON post_tags.tag = tags.id \
             WHERE post_tags.post = ?",
        )?;
        let rows = stmt.query_map([post], Tag::from_row)?;
        rows.collect()
    }

    /// Fetch all file metas of a post.
    pub(crate) fn list_post_files(&self, post: PostId) -> Result<Vec<FileMeta>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM file_metas WHERE post = ?")?;
        let rows = stmt.query_map([post], FileMeta::from_row)?;
        rows.collect()
    }

    /// Fetch all collections a post belongs to.
    pub(crate) fn list_post_collections(
        &self,
        post: PostId,
    ) -> Result<Vec<Collection>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "SELECT collections.* FROM collections \
             INNER JOIN collection_posts ON collection_posts.collection = collections.id \
             WHERE collection_posts.post = ?",
        )?;
        let rows = stmt.query_map([post], Collection::from_row)?;
        rows.collect()
    }
}
