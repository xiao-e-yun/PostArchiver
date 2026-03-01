//! Query builder module providing fluent read operations for all entities.
//!
//! # Entry points on [`PostArchiverManager`](crate::manager::PostArchiverManager)
//!
//! | Method | Builder | Returns |
//! |--------|---------|---------|
//! | `manager.posts()` | [`PostQuery`](post::PostQuery) | `Vec<Post>` / `PageResult<Post>` / `…WithRelations` |
//! | `manager.authors()` | [`AuthorQuery`](author::AuthorQuery) | `Vec<Author>` / `PageResult<Author>` / `…WithRelations` |
//! | `manager.tags()` | [`TagQuery`](tag::TagQuery) | `Vec<Tag>` / `PageResult<Tag>` |
//! | `manager.platforms()` | [`PlatformQuery`](platform::PlatformQuery) | `Vec<Platform>` |
//! | `manager.collections()` | [`CollectionQuery`](collection::CollectionQuery) | `Vec<Collection>` / `PageResult<Collection>` |
//!
//! # Typestate
//!
//! `.relations()` and `.with_total()` transition the builder's typestate at *compile time*,
//! changing the return type of `.query()` without any runtime cost.
//!
//! ```no_run
//! # use post_archiver::manager::PostArchiverManager;
//! # use post_archiver::query::post::PostSort;
//! # use post_archiver::query::SortDir;
//! # let manager = PostArchiverManager::open_in_memory().unwrap();
//! // Vec<Post>
//! let v = manager.posts().pagination(20, 0).query().unwrap();
//!
//! // PageResult<Post>  (adds `.total`)
//! let p = manager.posts().pagination(20, 0).with_total().query().unwrap();
//! println!("{} total posts", p.total);
//! ```

pub mod author;
pub mod collection;
pub mod file_meta;
pub mod platform;
pub mod post;
pub mod tag;

pub use author::{AuthorQuery, AuthorSort, AuthorWithRelations};
pub use collection::CollectionQuery;
pub use platform::PlatformQuery;
pub use post::{PostQuery, PostSort, PostWithRelations};
pub use tag::TagQuery;

// ── Pagination ────────────────────────────────────────────────────────────────

/// Pagination parameters. `page` is 0-based; SQL `OFFSET = page * limit`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Pagination {
    pub limit: u64,
    pub page: u64,
}

impl Pagination {
    pub fn new(limit: u64, page: u64) -> Self {
        Self { limit, page }
    }

    pub fn offset(self) -> u64 {
        self.page * self.limit
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Self { limit: 50, page: 0 }
    }
}

// ── Sort direction ────────────────────────────────────────────────────────────

/// Sort direction used with `.sort(field, dir)` builder methods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SortDir {
    #[default]
    Asc,
    Desc,
}

impl SortDir {
    pub(crate) fn as_sql(self) -> &'static str {
        match self {
            SortDir::Asc => "ASC",
            SortDir::Desc => "DESC",
        }
    }
}

// ── PageResult ────────────────────────────────────────────────────────────────

/// Result container for paginated queries (produced by `.with_total().query()`).
#[derive(Debug, Clone)]
pub struct PageResult<T> {
    pub items: Vec<T>,
    /// Total number of rows matching the filter, ignoring pagination.
    pub total: u64,
}

// ── Typestate markers ─────────────────────────────────────────────────────────

/// Typestate marker: `.query()` returns bare entities (no relations).
#[derive(Debug, Clone, Copy)]
pub struct NoRelations;

/// Typestate marker: `.query()` returns entities with all relations loaded.
#[derive(Debug, Clone, Copy)]
pub struct WithRelations;

/// Typestate marker: `.query()` returns a plain `Vec<T>` (no total count).
#[derive(Debug, Clone, Copy)]
pub struct NoTotal;

/// Typestate marker: `.query()` returns a [`PageResult<T>`] containing the total count.
#[derive(Debug, Clone, Copy)]
pub struct WithTotal;
