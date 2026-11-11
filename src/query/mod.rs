//! Query builder module providing fluent read-only operations for all entities.
//!
//! # Architecture
//!
//! Each query builder (e.g. [`PostQuery`](post::PostQuery)) exposes a set of public filter
//! fields (from the [`filter`] module: `IdFilter`, `TextFilter`, `DateFilter`,
//! `RelationshipsFilter`). Callers mutate those fields directly, then call
//! [`Query::query()`] to execute the SQL and retrieve results.
//!
//! Ordering and paging are plain fields on every builder:
//! - `order`: [`OrderBy`], keys apply in call order (`q.order.desc(..).asc(..)`, or `q.order.random()`)
//! - `pagination`: [`Pagination`], e.g. `Pagination::default().limit(20).page(1)`
//! - [`Query::count()`] returns the number of matching rows, ignoring `order` and `pagination`
//!
//! Single rows are fetched by ID with [`Repo::get()`], into any [`FromQuery`] type based on
//! that table (the entity itself, or a projection made with
//! [`impl_from_query!`](crate::impl_from_query)).
//!
//! # Entry points ([`PostArchiverQueryExt`], on the manager or any `Deref<Target = Connection>`)
//!
//! | Method | Builder from `.query()` |
//! |--------|-------------------------|
//! | `manager.posts()` | [`PostQuery`](post::PostQuery) |
//! | `manager.authors()` | [`AuthorQuery`](author::AuthorQuery) |
//! | `manager.tags()` | [`TagQuery`](tag::TagQuery) |
//! | `manager.platforms()` | [`PlatformQuery`](platform::PlatformQuery) |
//! | `manager.collections()` | [`CollectionQuery`](collection::CollectionQuery) |
//! | `manager.file_metas()` | [`FileMetaQuery`](file_meta::FileMetaQuery) |
//!
//! # Examples
//!
//! ```
//! # use post_archiver::manager::PostArchiverManager;
//! use post_archiver::query::{post::PostSort, Pagination, PostArchiverQueryExt, Query};
//! use post_archiver::{Post, PostId};
//! # let manager = PostArchiverManager::open_in_memory().unwrap();
//!
//! // newest posts, second page of 20, plus the total for the pager
//! let mut q = manager.posts().query();
//! q.title.contains("hello");
//! q.order.desc(PostSort::Updated);
//! q.pagination = Pagination::default().limit(20).page(1);
//! let total = q.count().unwrap();
//! let page: Vec<Post> = q.query().unwrap();
//!
//! // a single post by ID
//! let post: Option<Post> = manager.posts().get(PostId(1)).unwrap();
//! # assert_eq!((total, page.len(), post), (0, 0, None));
//! ```

pub mod author;
pub mod collection;
pub mod file_meta;
pub mod filter;
pub mod platform;
pub mod post;
pub mod tag;

use std::{
    fmt::{Debug, Display},
    ops::Deref,
};

use rusqlite::{params, Connection, OptionalExtension, Result, ToSql};

use crate::{utils::macros::AsTable, HasId};

pub trait PostArchiverQueryExt {
    fn posts(&'_ self) -> Repo<'_, crate::Post>;
    fn authors(&'_ self) -> Repo<'_, crate::Author>;
    fn tags(&'_ self) -> Repo<'_, crate::Tag>;
    fn collections(&'_ self) -> Repo<'_, crate::Collection>;
    fn platforms(&'_ self) -> Repo<'_, crate::Platform>;
    fn file_metas(&'_ self) -> Repo<'_, crate::FileMeta>;
}
impl<T> PostArchiverQueryExt for T
where
    T: Deref<Target = Connection>,
{
    fn posts(&'_ self) -> Repo<'_, crate::Post> {
        Repo::new(self)
    }
    fn authors(&'_ self) -> Repo<'_, crate::Author> {
        Repo::new(self)
    }
    fn tags(&'_ self) -> Repo<'_, crate::Tag> {
        Repo::new(self)
    }
    fn collections(&'_ self) -> Repo<'_, crate::Collection> {
        Repo::new(self)
    }
    fn platforms(&'_ self) -> Repo<'_, crate::Platform> {
        Repo::new(self)
    }
    fn file_metas(&'_ self) -> Repo<'_, crate::FileMeta> {
        Repo::new(self)
    }
}

pub struct Repo<'a, T> {
    pub conn: &'a Connection,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, T: AsTable + HasId> Repo<'a, T> {
    fn new(conn: &'a Connection) -> Repo<'a, T> {
        Self {
            conn,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn get<U: FromQuery<Based = T>>(&self, id: T::Id) -> crate::error::Result<Option<U>> {
        let sql = format!("SELECT {} FROM {} WHERE id = ?", U::fields(), T::TABLE_NAME);
        let mut stmt = self.conn.prepare_cached(&sql)?;
        Ok(stmt.query_row(params![id], U::from_row).optional()?)
    }
}

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
    fn fields() -> String;
    /// Deserializes one instance of this type from a rusqlite `Row`.
    fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error>;
}

// ── Query Trait ───────────────────────────────────────────────────────────────

pub trait Query: Sized {
    type Source: AsTable + HasId;
    fn conn(&self) -> &Connection;
    fn sql(&self, sql: SqlBuilder) -> SqlBuilder;

    fn query<T: FromQuery<Based = Self::Source>>(&self) -> crate::error::Result<Vec<T>> {
        let sql = SqlBuilder::table(Self::Source::TABLE_NAME).select(T::fields());
        let (sql, params) = self.sql(sql).build();
        let mut stmt = self.conn().prepare_cached(&sql)?;
        let params = params.iter().map(|p| p.as_ref()).collect::<Vec<_>>();
        let rows = stmt.query_map(params.as_slice(), T::from_row)?;
        Ok(rows.collect::<Result<_, _>>()?)
    }

    fn count(&self) -> crate::error::Result<u64> {
        let sql = SqlBuilder::table(Self::Source::TABLE_NAME).select("COUNT(*)".to_string());
        let mut sql = self.sql(sql);
        sql.pagination = Pagination::default();
        sql.order_clause = String::new();
        let (sql, params) = sql.build();
        let mut stmt = self.conn().prepare_cached(&sql)?;
        let params = params.iter().map(|p| p.as_ref()).collect::<Vec<_>>();
        Ok(stmt.query_row(params.as_slice(), |row| row.get(0))?)
    }
}

#[derive(Default)]
pub struct SqlBuilder {
    table: String,
    select: String,
    where_clause: Vec<String>,
    order_clause: String,
    pagination: Pagination,
    params: Vec<Param>,
}

impl SqlBuilder {
    pub fn table(table: &str) -> Self {
        Self {
            table: table.to_string(),
            ..Default::default()
        }
    }
    pub fn select(mut self, fields: String) -> Self {
        self.select = fields;
        self
    }

    pub fn filter<T: Filter>(self, filter: &T) -> Self {
        filter.apply(self)
    }

    pub fn order_by<T: Display>(mut self, order_by: &OrderBy<T>) -> Self {
        self.order_clause = order_by.extract();
        self
    }

    pub fn pagination(mut self, pagination: &Pagination) -> Self {
        self.pagination = *pagination;
        self
    }

    pub fn build(self) -> (String, Vec<Param>) {
        let mut params = self.params;
        let mut sql = vec![format!("SELECT {} FROM {}", self.select, self.table)];

        if !self.where_clause.is_empty() {
            sql.push(format!("WHERE {}", self.where_clause.join(" AND ")));
        }

        if !self.order_clause.is_empty() {
            sql.push(format!("ORDER BY {}", self.order_clause));
        }

        if let Some((clause, values)) = self.pagination.extract() {
            sql.push(clause);
            params.extend(values);
        }

        (sql.join(" "), params)
    }
}

pub trait Filter: Sized {
    fn apply(&self, sql: SqlBuilder) -> SqlBuilder;
}

pub type Param = Box<dyn ToSql>;

// ── Paginate ─────────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Pagination {
    limit: Option<u64>,
    offset: Option<u64>,
    is_page: bool,
}

impl Pagination {
    pub fn limit(mut self, limit: u64) -> Self {
        self.limit = Some(limit);
        self
    }
    pub fn offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self.is_page = false;
        self
    }
    pub fn page(mut self, page: u64) -> Self {
        self.offset = Some(page);
        self.is_page = true;
        self
    }
    pub fn extract(&self) -> Option<(String, Vec<Param>)> {
        match (self.limit, self.offset) {
            (None, None) => None,
            (Some(limit), None) => Some(("LIMIT ?".to_string(), vec![Box::new(limit) as Param])),
            (limit, Some(offset)) => Some((
                "LIMIT ? OFFSET ?".to_string(),
                vec![
                    match limit {
                        Some(l) => Box::new(l) as Param,
                        None => Box::new(-1) as Param,
                    },
                    Box::new(match self.is_page {
                        true => offset * limit.unwrap_or(0),
                        false => offset,
                    }) as Param,
                ],
            )),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OrderBy<T> {
    clauses: Vec<String>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> OrderBy<T> {
    pub fn random(&mut self) -> &mut Self {
        self.clauses.push("RANDOM()".to_string());
        self
    }

    pub fn extract(&self) -> String
    where
        T: Display,
    {
        self.clauses.join(", ")
    }
}

impl<T> Default for OrderBy<T> {
    fn default() -> Self {
        Self {
            clauses: vec![],
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T: SortField> OrderBy<T> {
    pub fn asc(&mut self, field: T) -> &mut Self {
        self.clauses.push(format!("{} ASC", field));
        self
    }

    pub fn desc(&mut self, field: T) -> &mut Self {
        self.clauses.push(format!("{} DESC", field));
        self
    }
}

pub trait SortField: Display {}

macro_rules! impl_sortable {
    ($sort_field_enum:ident {
        $($field:ident: $column:literal),*
    }) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        pub enum $sort_field_enum {
            $($field),*
        }

        impl $crate::query::SortField for $sort_field_enum {}
        impl std::fmt::Display for $sort_field_enum {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    $(Self::$field => write!(f, $column)),*
                }
            }
        }
    };
}
pub(crate) use impl_sortable;
