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
//! let v = manager.posts().query::<post_archiver::Post>().unwrap();
//!
//! // Vec<Post>  (paginated)
//! let v = manager.posts().pagination(20, 0).query::<post_archiver::Post>().unwrap();
//!
//! // PageResult<Post>  (paginated + total count)
//! let p = manager.posts().pagination(20, 0).with_total().query::<post_archiver::Post>().unwrap();
//! println!("{} total posts", p.total);
//! ```

pub mod author;
pub mod collection;
pub mod file_meta;
pub mod filter;
pub mod platform;
pub mod post;
pub mod tag;

use cached::Cached;
pub use countable::{Countable, Totalled};
pub use paginate::Paginate;
pub use sortable::{SortDir, Sortable};

use std::{
    fmt::{Debug, Display},
    rc::Rc,
};

use rusqlite::ToSql;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
};

pub trait FromQuery: Sized {
    type Based: AsTable;
    fn select_sql() -> String;
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>;
}

// ── Query Trait ───────────────────────────────────────────────────────────────

/// Core query execution trait. Implementors provide `.query()` to fetch results.
pub trait Query: Sized {
    type Wrapper<T>;
    type Based: AsTable;
    /// Execute the query, returning matching items.
    fn query_with_context<T: FromQuery<Based = Self::Based>>(
        self,
        sql: RawSql<T>,
    ) -> crate::error::Result<Self::Wrapper<T>>;

    fn query<T: FromQuery<Based = Self::Based>>(self) -> crate::error::Result<Self::Wrapper<T>> {
        let sql: RawSql<T> = RawSql::new();
        self.query_with_context(sql)
    }
}

pub trait BaseFilter: Sized {
    type Based: AsTable;
    fn update_sql<T: FromQuery<Based = Self::Based>>(&self, sql: RawSql<T>) -> RawSql<T>;
    fn queryer(&self) -> &Queryer<'_, impl PostArchiverConnection>;

    fn count(&self) -> crate::error::Result<u64> {
        let sql = RawSql::<Self::Based>::new();
        let sql = self.update_sql(sql);
        let (sql, params) = sql.build_count_sql();

        let cache_key = (
            sql.clone(),
            params
                .iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(","),
        );

        let cached_count = self
            .queryer()
            .manager
            .caches
            .lock()
            .unwrap()
            .counts
            .cache_get(&cache_key)
            .cloned();
        match cached_count {
            Some(cached_count) => Ok(cached_count),
            None => {
                let count = self.queryer().count(&sql, params)?;
                self.queryer()
                    .manager
                    .caches
                    .lock()
                    .unwrap()
                    .counts
                    .cache_set(cache_key, count);
                Ok(count)
            }
        }
    }
}

pub trait ToSqlAndEq: ToSql + Display {}
impl<T: ToSql + Display> ToSqlAndEq for T {}

pub type Param = Rc<dyn ToSqlAndEq>;

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

    pub fn build_generic_sql(&self) -> (String, Vec<Param>) {
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
        (format!("{} {} {}", where_sql, order_sql, limit_sql), params)
    }
}

impl<T: FromQuery> RawSql<T> {
    pub fn build_sql(&self) -> (String, Vec<Param>) {
        let (clause, params) = self.build_generic_sql();
        let sql = format!("{} {}", T::select_sql(), clause);
        (sql, params)
    }

    pub fn build_count_sql(&self) -> (String, Vec<Param>) {
        let where_sql = if self.where_clause.0.is_empty() {
            "".to_string()
        } else {
            format!("WHERE {}", self.where_clause.0.join(" AND "))
        };
        let sql = format!(
            "SELECT COUNT(*) FROM {} {}",
            T::Based::TABLE_NAME,
            where_sql
        );
        (sql, self.where_clause.1.clone())
    }
}

pub struct Queryer<'a, C> {
    pub manager: &'a PostArchiverManager<C>,
}

impl<T> Debug for Queryer<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Queryer").finish()
    }
}

impl<'a, C: PostArchiverConnection> Queryer<'a, C> {
    pub fn new(manager: &'a PostArchiverManager<C>) -> Self {
        Self { manager }
    }

    pub fn fetch<Q: FromQuery>(
        &self,
        sql: &str,
        params: Vec<Param>,
    ) -> crate::error::Result<Vec<Q>> {
        let mut stmt = self.manager.conn().prepare_cached(sql)?;
        let params = params
            .iter()
            .map(|p| p.as_ref() as &dyn ToSql)
            .collect::<Vec<_>>();
        let rows = stmt.query_map(params.as_slice(), Q::from_row)?;
        rows.collect::<Result<_, _>>().map_err(Into::into)
    }

    pub fn count(&self, sql: &str, params: Vec<Param>) -> crate::error::Result<u64> {
        let mut stmt = self.manager.conn().prepare_cached(sql)?;
        let params = params
            .iter()
            .map(|p| p.as_ref() as &dyn ToSql)
            .collect::<Vec<_>>();
        Ok(stmt.query_row(params.as_slice(), |row| row.get(0))?)
    }
}

// ── Paginate ─────────────────────────────────────────────────────────────────
pub use paginate::*;
pub mod paginate {
    use crate::{
        manager::PostArchiverConnection,
        query::{Query, RawSql},
    };

    use super::{BaseFilter, FromQuery};

    pub trait Paginate: Sized {
        fn pagination(self, limit: u64, page: u64) -> Paginated<Self>;
    }

    impl<T: Query> Paginate for T {
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

    impl<Q: Query> Query for Paginated<Q> {
        type Wrapper<T> = Q::Wrapper<T>;
        type Based = Q::Based;

        fn query_with_context<T: FromQuery<Based = Self::Based>>(
            self,
            mut sql: RawSql<T>,
        ) -> crate::error::Result<Self::Wrapper<T>> {
            sql.limit_clause = Some([self.limit, self.limit * self.page]);
            self.inner.query_with_context(sql)
        }
    }

    impl<Q: BaseFilter> BaseFilter for Paginated<Q> {
        type Based = Q::Based;

        fn update_sql<T: FromQuery<Based = Self::Based>>(&self, sql: RawSql<T>) -> RawSql<T> {
            self.inner.update_sql(sql)
        }

        fn queryer(&self) -> &crate::query::Queryer<'_, impl PostArchiverConnection> {
            self.inner.queryer()
        }
    }
}

pub use countable::*;
pub mod countable {
    use serde::{Deserialize, Serialize};

    use crate::{
        manager::PostArchiverConnection,
        query::{Query, RawSql},
    };

    use super::{BaseFilter, FromQuery};

    pub trait Countable: Sized {
        /// Chain: wrap result in [`PageResult`] including total count.
        fn with_total(self) -> WithTotal<Self> {
            WithTotal { inner: self }
        }
    }

    impl<T: Query> Countable for T {}

    #[derive(Debug)]
    pub struct WithTotal<Q> {
        inner: Q,
    }

    impl<Q: Query + BaseFilter> Query for WithTotal<Q> {
        type Wrapper<T> = Totalled<Q::Wrapper<T>>;
        type Based = <Q as Query>::Based;

        fn query_with_context<T: FromQuery<Based = Self::Based>>(
            self,
            sql: RawSql<T>,
        ) -> crate::error::Result<Self::Wrapper<T>> {
            let total = self.inner.count()?;
            let items = self.inner.query_with_context(sql)?;
            Ok(Totalled { items, total })
        }
    }

    impl<T: BaseFilter> BaseFilter for WithTotal<T> {
        type Based = T::Based;

        fn update_sql<U: FromQuery<Based = Self::Based>>(&self, sql: RawSql<U>) -> RawSql<U> {
            self.inner.update_sql(sql)
        }

        fn queryer(&self) -> &crate::query::Queryer<'_, impl PostArchiverConnection> {
            self.inner.queryer()
        }
    }

    /// Result container for paginated queries (produced by `.with_total().query()`).
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Totalled<T> {
        pub items: T,
        /// Total number of rows matching the filter, ignoring pagination.
        pub total: u64,
    }
}

pub use sortable::*;
pub mod sortable {
    use std::fmt::Display;

    use crate::{
        manager::PostArchiverConnection,
        query::{Query, RawSql},
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
        pub fn as_sql(self) -> &'static str {
            match self {
                SortDir::Asc => "ASC",
                SortDir::Desc => "DESC",
            }
        }
    }

    impl<Q: Query, U: Display> Query for Sorted<Q, U> {
        type Wrapper<T> = Q::Wrapper<T>;
        type Based = Q::Based;
        fn query_with_context<T: FromQuery<Based = Self::Based>>(
            self,
            mut sql: RawSql<T>,
        ) -> crate::error::Result<Self::Wrapper<T>> {
            sql.order_clause
                .push(format!("{} {}", self.field, self.dir.as_sql()));
            self.inner.query_with_context(sql)
        }
    }

    impl<Q: Query> Query for Sorted<Q, Random> {
        type Wrapper<T> = Q::Wrapper<T>;
        type Based = Q::Based;
        fn query_with_context<T: FromQuery<Based = Self::Based>>(
            self,
            mut sql: RawSql<T>,
        ) -> crate::error::Result<Self::Wrapper<T>> {
            sql.order_clause.push("RANDOM()".to_string());
            self.inner.query_with_context(sql)
        }
    }

    impl<Q: BaseFilter, U> BaseFilter for Sorted<Q, U> {
        type Based = Q::Based;

        fn update_sql<T: FromQuery<Based = Self::Based>>(&self, sql: RawSql<T>) -> RawSql<T> {
            self.inner.update_sql(sql)
        }

        fn queryer(&self) -> &super::Queryer<'_, impl PostArchiverConnection> {
            self.inner.queryer()
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

    #[macro_export]
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
    pub use impl_sortable;

    use super::{countable::WithTotal, paginate::Paginated, BaseFilter, FromQuery};
}
