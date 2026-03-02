//! Query builder module providing fluent read operations for all entities.
//!
//! # Entry points on [`PostArchiverManager`](crate::manager::PostArchiverManager)
//!
//! | Method | Builder | Returns |
//! |--------|---------|---------|
//! | `manager.posts()` | [`PostQuery`](post::PostQuery) | `Vec<Post>` / `PageResult<Post>` |
//! | `manager.authors()` | [`AuthorQuery`](author::AuthorQuery) | `Vec<Author>` / `PageResult<Author>` |
//! | `manager.tags()` | [`TagQuery`](tag::TagQuery) | `Vec<Tag>` / `PageResult<Tag>` |
//! | `manager.platforms()` | [`PlatformQuery`](platform::PlatformQuery) | `Vec<Platform>` |
//! | `manager.collections()` | [`CollectionQuery`](collection::CollectionQuery) | `Vec<Collection>` / `PageResult<Collection>` |
//!
//! # Chain Style
//!
//! `.pagination()` wraps the builder in [`Paginated`], and `.with_total()` further
//! wraps it to include the total count.
//!
//! ```no_run
//! # use post_archiver::manager::PostArchiverManager;
//! # use post_archiver::query::{Countable, Paginate, Query};
//! # let manager = PostArchiverManager::open_in_memory().unwrap();
//! // Vec<Post>  (all matching, no LIMIT)
//! let v = manager.posts().query().unwrap();
//!
//! // Vec<Post>  (paginated)
//! let v = manager.posts().pagination(20, 0).query().unwrap();
//!
//! // PageResult<Post>  (paginated + total count)
//! let p = manager.posts().pagination(20, 0).with_total().query().unwrap();
//! println!("{} total posts", p.total);
//! ```

pub mod author;
pub mod collection;
pub mod file_meta;
pub mod filter;
pub mod platform;
pub mod post;
pub mod tag;

use std::{fmt::Debug, rc::Rc};

use rusqlite::ToSql;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
};

// ── Query Trait ───────────────────────────────────────────────────────────────

/// Core query execution trait. Implementors provide `.query()` to fetch results.
pub trait Query: Sized + RawQuery {
    type Wrapper<T>;
    /// Execute the query, returning matching items.
    fn query_with_context(
        self,
        sql: &str,
        params: Vec<Param>,
    ) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error>;

    fn query(self) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error> {
        let (sql, params) = self.sql().build_sql();
        self.query_with_context(&sql, params)
    }
}

// ── RawQuery (sealed) ────────────────────────────────────────────────────────

/// Low-level SQL builder trait used by [`Paginated`].
///
/// **Sealed** — cannot be implemented outside this crate.
pub(crate) trait RawQuery: Sized {
    /// Output item type.
    type Item: AsTable;
    /// Build the raw SQL query.
    #[doc(hidden)]
    fn sql(&self) -> RawSql<Self::Item>;
    fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection>;
}

type Param = Rc<dyn ToSql>;

#[derive(Default, Clone)]
pub struct RawSql<T> {
    pub where_clause: (Vec<String>, Vec<Param>),
    pub order_clause: Vec<String>,
    pub limit_clause: Option<[u64; 2]>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> Debug for RawSql<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RawSql")
            .field("where_clause", &self.where_clause.0)
            .field("order_clause", &self.order_clause)
            .field("limit_clause", &self.limit_clause)
            .finish()
    }
}

impl<T> RawSql<T> {
    pub fn new() -> Self {
        Self {
            where_clause: (Vec::new(), Vec::new()),
            order_clause: Vec::new(),
            limit_clause: None,
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: AsTable> RawSql<T> {
    pub fn build_sql(&self) -> (String, Vec<Param>) {
        let mut params = self.where_clause.1.clone();
        let where_sql = if self.where_clause.0.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", self.where_clause.0.join(" AND "))
        };
        let order_sql = if self.order_clause.is_empty() {
            "".to_string()
        } else {
            format!("ORDER BY {}", self.order_clause.join(", "))
        };
        let limit_sql = if let Some([limit, offset]) = self.limit_clause {
            params.push(Rc::new(limit));
            params.push(Rc::new(offset));
            "LIMIT ? OFFSET ?".to_string()
        } else {
            "".to_string()
        };
        let sql = format!(
            "SELECT * FROM {} {} {} {}",
            T::TABLE_NAME,
            where_sql,
            order_sql,
            limit_sql
        );
        (sql, params)
    }

    pub fn build_count_sql(&self) -> (String, Vec<Param>) {
        let where_sql = if self.where_clause.0.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", self.where_clause.0.join(" AND "))
        };
        let sql = format!("SELECT COUNT(*) FROM {} {}", T::TABLE_NAME, where_sql);
        (sql, self.where_clause.1.clone())
    }
}

pub struct Queryer<'a, C> {
    pub(crate) manager: &'a PostArchiverManager<C>,
}

impl<T> Debug for Queryer<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queryer").finish()
    }
}

impl<'a, C: PostArchiverConnection> Queryer<'a, C> {
    pub(crate) fn new(manager: &'a PostArchiverManager<C>) -> Self {
        Self { manager }
    }

    pub fn fetch<Q: AsTable>(
        &self,
        sql: &str,
        params: Vec<Param>,
    ) -> Result<Vec<Q>, rusqlite::Error> {
        let mut stmt = self.manager.conn().prepare_cached(sql)?;
        let params = params
            .iter()
            .map(|p| p.as_ref() as &dyn ToSql)
            .collect::<Vec<_>>();
        let rows = stmt.query_map(params.as_slice(), Q::from_row)?;
        rows.collect()
    }

    pub fn count(&self, sql: &str, params: Vec<Param>) -> Result<u64, rusqlite::Error> {
        let mut stmt = self.manager.conn().prepare_cached(sql)?;
        let params = params
            .iter()
            .map(|p| p.as_ref() as &dyn ToSql)
            .collect::<Vec<_>>();
        stmt.query_row(params.as_slice(), |row| row.get(0))
    }
}

// ── Paginate ─────────────────────────────────────────────────────────────────
pub mod paginate {
    use crate::{
        manager::PostArchiverConnection,
        query::{Query, RawQuery, RawSql},
    };

    use super::Queryer;

    pub trait Paginate: Sized {
        fn pagination(self, limit: u64, page: u64) -> Paginated<Self>;
    }

    impl<T: RawQuery> Paginate for T {
        fn pagination(self, limit: u64, page: u64) -> Paginated<Self> {
            Paginated {
                inner: self,
                limit,
                page,
            }
        }
    }

    /// Paginated wrapper produced by [`.pagination()`](Paginate::pagination).
    ///
    /// Appends `LIMIT … OFFSET …` to the inner builder's SQL.
    #[derive(Debug)]
    pub struct Paginated<Q> {
        inner: Q,
        limit: u64,
        page: u64,
    }

    impl<Q: RawQuery> RawQuery for Paginated<Q> {
        type Item = Q::Item;
        fn sql(&self) -> RawSql<Self::Item> {
            let mut raw_sql = self.inner.sql();
            raw_sql.limit_clause = Some([self.limit, (self.page * self.limit)]);
            raw_sql
        }

        fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
            self.inner.queryer()
        }
    }

    impl<Q: Query> Query for Paginated<Q> {
        type Wrapper<T> = Q::Wrapper<T>;
        fn query_with_context(
            self,
            sql: &str,
            params: Vec<super::Param>,
        ) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error> {
            self.inner.query_with_context(sql, params)
        }
    }
}

// ── Countable Trait ───────────────────────────────────────────────────────────

pub mod countable {
    use crate::{
        manager::PostArchiverConnection,
        query::{Query, RawQuery, RawSql},
    };

    use super::Queryer;

    pub trait Countable: Sized {
        /// Count total matching rows (ignoring pagination).
        fn count(&self) -> Result<u64, rusqlite::Error>;

        /// Chain: wrap result in [`PageResult`] including total count.
        fn with_total(self) -> WithTotal<Self> {
            WithTotal(self)
        }
    }

    impl<T: RawQuery> Countable for T {
        fn count(&self) -> Result<u64, rusqlite::Error> {
            let (sql, params) = self.sql().build_count_sql();
            self.queryer().count(&sql, params)
        }
    }

    // ── WithTotal wrapper ─────────────────────────────────────────────────────────

    #[derive(Debug)]
    pub struct WithTotal<Q>(Q);

    impl<Q: RawQuery> RawQuery for WithTotal<Q> {
        type Item = Q::Item;
        fn sql(&self) -> RawSql<Self::Item> {
            self.0.sql()
        }
        fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
            self.0.queryer()
        }
    }

    impl<Q: Query + Countable> Query for WithTotal<Q> {
        type Wrapper<T> = PageResult<Q::Wrapper<T>>;
        fn query_with_context(
            self,
            sql: &str,
            params: Vec<super::Param>,
        ) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error> {
            let total = self.0.count()?;
            let items = self.0.query_with_context(sql, params)?;
            Ok(PageResult { items, total })
        }
    }

    /// Result container for paginated queries (produced by `.with_total().query()`).
    #[derive(Debug, Clone)]
    pub struct PageResult<T> {
        pub items: T,
        /// Total number of rows matching the filter, ignoring pagination.
        pub total: u64,
    }
}

pub mod sortable {
    use std::fmt::Display;

    use crate::{
        manager::PostArchiverConnection,
        query::{Query, RawQuery, RawSql},
    };

    pub trait Sortable: Sized {
        type SortField;
        fn sort(self, field: Self::SortField, dir: SortDir) -> Sorted<Self, Self::SortField> {
            Sorted {
                inner: self,
                field,
                dir,
            }
        }
        fn sort_random(self) -> Sorted<Self, Random> {
            Sorted {
                inner: self,
                field: Random,
                dir: SortDir::Asc, // ignored
            }
        }
    }

    #[derive(Debug)]
    pub struct Sorted<Q, F> {
        inner: Q,
        field: F,
        dir: SortDir,
    }

    #[derive(Debug)]
    pub struct Random;

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

    impl<Q: RawQuery, U: Display> RawQuery for Sorted<Q, U> {
        type Item = Q::Item;
        fn sql(&self) -> RawSql<Self::Item> {
            let mut raw_sql = self.inner.sql();
            raw_sql
                .order_clause
                .push(format!("{} {}", self.field, self.dir.as_sql()));
            raw_sql
        }
        fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
            self.inner.queryer()
        }
    }

    impl<Q: RawQuery> RawQuery for Sorted<Q, Random> {
        type Item = Q::Item;
        fn sql(&self) -> RawSql<Self::Item> {
            let mut raw_sql = self.inner.sql();
            raw_sql.order_clause.push("RANDOM()".to_string());
            raw_sql
        }
        fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection> {
            self.inner.queryer()
        }
    }

    impl<Q: Query, U: Display> Query for Sorted<Q, U> {
        type Wrapper<T> = Q::Wrapper<T>;
        fn query_with_context(
            self,
            sql: &str,
            params: Vec<super::Param>,
        ) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error> {
            self.inner.query_with_context(sql, params)
        }
    }

    impl<Q: Query> Query for Sorted<Q, Random> {
        type Wrapper<T> = Q::Wrapper<T>;
        fn query_with_context(
            self,
            sql: &str,
            params: Vec<super::Param>,
        ) -> Result<Self::Wrapper<Self::Item>, rusqlite::Error> {
            self.inner.query_with_context(sql, params)
        }
    }

    impl<T: Sortable> Sortable for WithTotal<T> {
        type SortField = T::SortField;
    }
    impl<T: Sortable> Sortable for Paginated<T> {
        type SortField = T::SortField;
    }
    impl<T: Sortable, U> Sortable for Sorted<T, U> {
        type SortField = T::SortField;
    }

    macro_rules! impl_sortable {
        ($query_type:ident ($sort_field_enum:ident) {
            $($field:ident: $column:expr),*
        }) => {
            pub enum $sort_field_enum {
                $($field),*
            }

            impl std::fmt::Display for $sort_field_enum {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    match self {
                        $(Self::$field => write!(f, "{}", $column)),*
                    }
                }
            }

            impl<C> $crate::query::sortable::Sortable for $query_type<'_, C> {
                type SortField = $sort_field_enum;
            }
        };
    }
    pub(crate) use impl_sortable;

    use super::{countable::WithTotal, paginate::Paginated, Queryer};
}
