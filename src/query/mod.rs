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

use std::fmt::{Debug, Display};

use rusqlite::{params, Connection, OptionalExtension, Result, ToSql};

use crate::{manager::PostArchiverManager, utils::macros::AsTable, HasId};

impl PostArchiverManager {
    pub fn posts(&self) -> Repo<'_, crate::Post> {
        Repo::new(&self.conn)
    }
    pub fn authors(&self) -> Repo<'_, crate::Author> {
        Repo::new(&self.conn)
    }
    pub fn tags(&self) -> Repo<'_, crate::Tag> {
        Repo::new(&self.conn)
    }
    pub fn collections(&self) -> Repo<'_, crate::Collection> {
        Repo::new(&self.conn)
    }
    pub fn platforms(&self) -> Repo<'_, crate::Platform> {
        Repo::new(&self.conn)
    }
    pub fn file_metas(&self) -> Repo<'_, crate::FileMeta> {
        Repo::new(&self.conn)
    }
}

pub struct Repo<'a, T> {
    pub conn: &'a Connection,
    _phantom: std::marker::PhantomData<T>,
}

impl<'a, E: AsTable + HasId> Repo<'a, E> {
    fn new(conn: &'a Connection) -> Repo<'a, E> {
        Self {
            conn,
            _phantom: std::marker::PhantomData,
        }
    }
    pub fn get<T: FromQuery<Based = E>>(&self, id: E::Id) -> crate::error::Result<Option<T>> {
        let sql = format!("SELECT {} FROM {} WHERE id = ?", T::fields(), E::TABLE_NAME);
        let mut stmt = self.conn.prepare_cached(&sql)?;
        Ok(stmt.query_row(params![id], T::from_row).optional()?)
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
}

impl Pagination {
    pub fn extract(&self) -> Option<(String, Vec<Param>)> {
        match (self.limit, self.offset) {
            (None, None) => None,
            (Some(limit), None) => Some(("LIMIT ?".to_string(), vec![Box::new(limit) as Param])),
            (limit, Some(offset)) => Some((
                "LIMIT ? OFFSET ?".to_string(),
                vec![
                    Box::new(limit.unwrap_or(u64::MAX)) as Param,
                    Box::new(offset) as Param,
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

#[macro_export]
macro_rules! impl_sortable {
    ($query_type:ident ($sort_field_enum:ident) {
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
pub use impl_sortable;
