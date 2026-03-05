//! Query builder module providing fluent read-only operations for all entities.
//!
//! # Architecture
//!
//! Each query builder (e.g. [`PostQuery`](post::PostQuery)) exposes a set of public filter
//! fields (from the [`filter`] module: `IdFilter`, `TextFilter`, `DateFilter`,
//! `RelationshipsFilter`). Callers mutate those fields directly, then call
//! [`Query::query()`] to execute the SQL and retrieve results.
//!
//! Builders can be composed via decorator wrappers:
//! - [`Paginate::pagination(limit, page)`](Paginate::pagination) → [`Paginated<Q>`](Paginated): appends `LIMIT … OFFSET …`
//! - [`Sortable::sort(field, dir)`](Sortable::sort) → [`Sorted<Q, F>`](sortable::Sorted): appends `ORDER BY …`
//! - [`Sortable::sort_random()`](Sortable::sort_random): appends `ORDER BY RANDOM()`
//! - [`Countable::with_total()`](Countable::with_total) → [`WithTotal<Q>`](countable::WithTotal): also returns the total row count
//!
//! # Entry points on [`PostArchiverManager`](crate::manager::PostArchiverManager)
//!
//! | Method | Builder | Returns |
//! |--------|---------|----------|
//! | `manager.posts()` | [`PostQuery`](post::PostQuery) | `Vec<Post>` / `Totalled<Vec<Post>>` |
//! | `manager.authors()` | [`AuthorQuery`](author::AuthorQuery) | `Vec<Author>` / `Totalled<Vec<Author>>` |
//! | `manager.tags()` | [`TagQuery`](tag::TagQuery) | `Vec<Tag>` / `Totalled<Vec<Tag>>` |
//! | `manager.platforms()` | [`PlatformQuery`](platform::PlatformQuery) | `Vec<Platform>` |
//! | `manager.collections()` | [`CollectionQuery`](collection::CollectionQuery) | `Vec<Collection>` / `Totalled<Vec<Collection>>` |
//!
//! # Examples
//!
//! ```no_run
//! # use post_archiver::manager::PostArchiverManager;
//! # use post_archiver::query::{Countable, Paginate, Query};
//! # let manager = PostArchiverManager::open_in_memory().unwrap();
//! // All matching posts (no LIMIT)
//! let v = manager.posts().query::<post_archiver::Post>().unwrap();
//!
//! // Paginated — page 0, 20 items per page
//! let v = manager.posts().pagination(20, 0).query::<post_archiver::Post>().unwrap();
//!
//! // Paginated + total count
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

/// Trait for types that can be deserialized from a SQL row.
///
/// Implement this trait to make a type queryable as a generic parameter of [`Query::query()`].
/// Typically implemented via `#[derive]` macros or manually on entity structs
/// (e.g. [`Post`](crate::Post), [`Author`](crate::Author)).
///
/// # Associated types
/// - `Based`: the corresponding database table type, must implement [`AsTable`].
pub trait FromQuery: Sized {
    type Based: AsTable;
    /// Returns the `SELECT …` SQL fragment used to query this type (without WHERE/ORDER/LIMIT).
    fn select_sql() -> String;
    /// Deserializes one instance of this type from a rusqlite `Row`.
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>;
}

// ── Query Trait ───────────────────────────────────────────────────────────────

/// Core query execution trait. Implementors can call `.query()` to run SQL and retrieve results.
///
/// You generally do not implement this trait directly; use the concrete query builders
/// (e.g. [`PostQuery`](post::PostQuery), [`AuthorQuery`](author::AuthorQuery)).
///
/// The decorator wrappers [`Paginated`], [`WithTotal`](countable::WithTotal), and
/// [`Sorted`](sortable::Sorted) all implement this trait by wrapping and delegating.
pub trait Query: Sized {
    /// The wrapper type for query results. For most builders this is `Vec<T>`;
    /// when wrapped by [`WithTotal`](countable::WithTotal) it becomes [`Totalled<Vec<T>>`](Totalled).
    type Wrapper<T>;
    /// The database table type this query targets.
    type Based: AsTable;
    /// Execute the query with an externally supplied [`RawSql`] context
    /// (typically threaded through decorator layers).
    fn query_with_context<T: FromQuery<Based = Self::Based>>(
        self,
        sql: RawSql<T>,
    ) -> crate::error::Result<Self::Wrapper<T>>;

    /// Execute the query with a default empty [`RawSql`] context, returning all matching results.
    fn query<T: FromQuery<Based = Self::Based>>(self) -> crate::error::Result<Self::Wrapper<T>> {
        let sql: RawSql<T> = RawSql::new();
        self.query_with_context(sql)
    }
}

/// Trait for types that hold filter conditions and can write them into a [`RawSql`].
///
/// Query builders (e.g. [`PostQuery`](post::PostQuery)) and decorator wrappers (e.g. [`Paginated`])
/// implement this trait so that [`BaseFilter::count()`] can compute the total matching row count.
/// The default `count()` implementation caches its result to avoid redundant queries.
pub trait BaseFilter: Sized {
    type Based: AsTable;
    /// Append all filter conditions held by this builder into `sql` and return the updated [`RawSql`].
    fn update_sql<T: FromQuery<Based = Self::Based>>(&self, sql: RawSql<T>) -> RawSql<T>;
    /// Return a reference to the [`Queryer`] owned by this builder, used by the default `count()` impl.
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

/// Intermediate SQL representation passed through the decorator chain and filled layer by layer.
///
/// Received by [`Query::query_with_context()`]; filters append WHERE conditions via
/// [`BaseFilter::update_sql()`]; [`Paginated`] sets `limit_clause`;
/// [`Sorted`](sortable::Sorted) appends to `order_clause`. Finally assembled into a
/// complete SQL string by [`RawSql::build_sql()`].
#[derive(Default, Clone)]
pub struct RawSql<T> {
    /// WHERE clause: `(list of condition strings, list of bound parameters)`, joined with `AND`.
    pub where_clause: (Vec<String>, Vec<Param>),
    /// ORDER BY expressions accumulated in append order.
    pub order_clause: Vec<String>,
    /// `[limit, offset]` corresponding to `LIMIT ? OFFSET ?`. `None` means no pagination.
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
    /// Create an empty [`RawSql`] with all fields at their defaults (no WHERE / ORDER / LIMIT).
    pub fn new() -> Self {
        Self {
            where_clause: (Vec::new(), Vec::new()),
            order_clause: Vec::new(),
            limit_clause: None,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Build the `WHERE … ORDER BY … LIMIT ? OFFSET ?` fragment (without the SELECT prefix).
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
    /// Build the full `SELECT … WHERE … ORDER BY … LIMIT ? OFFSET ?` SQL string.
    pub fn build_sql(&self) -> (String, Vec<Param>) {
        let (clause, params) = self.build_generic_sql();
        let sql = format!("{} {}", T::select_sql(), clause);
        (sql, params)
    }

    /// Build a `SELECT COUNT(*) FROM <table> WHERE …` SQL string (ORDER/LIMIT are ignored).
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

/// Helper that wraps a [`PostArchiverManager`] reference and provides low-level SQL execution.
///
/// Each query builder owns a `Queryer` at construction time and uses it to
/// access the database connection and result cache.
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

    /// Execute a SELECT query and deserialize all result rows into `Vec<Q>`.
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

    /// Execute a `SELECT COUNT(*)` query and return the number of matching rows.
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

    /// Trait that adds pagination support to query builders.
    ///
    /// Automatically implemented for all types that implement [`Query`](crate::query::Query).
    pub trait Paginate: Sized {
        /// Wrap this builder with `limit` (items per page) and `page` (0-based page index),
        /// returning [`Paginated<Self>`] which appends `LIMIT limit OFFSET limit*page` on execution.
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
    /// Appends `LIMIT … OFFSET …` to the inner builder's SQL on execution.
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

    /// Trait that adds total-count support to query builders.
    ///
    /// Automatically implemented for all types that implement [`Query`](crate::query::Query).
    pub trait Countable: Sized {
        /// Wrap this builder in [`WithTotal<Self>`]. When [`Query::query()`] is called,
        /// an additional `COUNT(*)` query is executed and the result is placed in
        /// the `total` field of the returned [`Totalled`].
        fn with_total(self) -> WithTotal<Self> {
            WithTotal { inner: self }
        }
    }

    impl<T: Query> Countable for T {}

    /// Wrapper produced by [`.with_total()`](Countable::with_total).
    /// Executes an additional `COUNT(*)` query and places the result in [`Totalled::total`].
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

    /// Result container produced by `.with_total().query()`, holding both the query results
    /// and the total count of rows matching the filter.
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
    pub struct Totalled<T> {
        /// The results retrieved by this query (respecting any LIMIT/OFFSET).
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

    /// Trait that adds sorting support to query builders.
    ///
    /// Per-entity sort-field enums (e.g. `PostSort`, `AuthorSort`) are generated
    /// automatically by the [`impl_sortable!`] macro.
    pub trait Sortable: Sized {
        /// The sortable field enum type, defined in each sub-module by [`impl_sortable!`].
        type SortField;
        /// Sort by the given field and direction, returning [`Sorted<Self, SortField>`].
        fn sort(self, field: Self::SortField, dir: SortDir) -> Sorted<Self, Self::SortField> {
            Sorted {
                inner: self,
                field,
                dir,
            }
        }
        /// Sort results randomly using `ORDER BY RANDOM()`.
        fn sort_random(self) -> Sorted<Self, Random> {
            Sorted {
                inner: self,
                field: Random,
                dir: SortDir::Asc, // ignored
            }
        }
    }

    /// Sorting wrapper produced by [`.sort()`](Sortable::sort), holding the sort field and direction.
    #[derive(Debug)]
    pub struct Sorted<Q, F> {
        inner: Q,
        field: F,
        dir: SortDir,
    }

    /// Marker type used by [`Sortable::sort_random()`] for random ordering.
    #[derive(Debug)]
    pub struct Random;

    /// Sort direction used with [`.sort(field, dir)`](Sortable::sort).
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub enum SortDir {
        /// Ascending order (default).
        #[default]
        Asc,
        /// Descending order.
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
