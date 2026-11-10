//! Query filters: reusable SQL WHERE condition builder stamps.
//!
//! Each query builder (e.g. [`PostQuery`](super::post::PostQuery)) exposes its filters
//! as public fields. Callers invoke methods on those fields; when the query runs, every
//! filter adds its `WHERE` condition and parameters to the [`SqlBuilder`] through
//! [`Filter::apply`]. Filters that were never set add nothing.
//!
//! | Filter | Use case |
//! |--------|----------|
//! | [`TextFilter`] | `LIKE` fuzzy matching on string columns |
//! | [`DateFilter`] | Date range filtering on `DateTime<Utc>` columns |
//! | [`IdFilter`] | Exact ID matching (`= ?` or `IN (…)`) |
//! | [`RelationshipsFilter`] | All-of matching across a many-to-many join table |

use std::{collections::HashSet, fmt::Display, hash::Hash, ops::Deref};

use chrono::{DateTime, Utc};
use rusqlite::ToSql;
use serde::Serialize;

use crate::query::{Filter, SqlBuilder};

/// SQL `LIKE` filter for string columns.
///
/// After specifying the target column, set the match pattern via one of the methods below.
/// Special characters `%` and `_` are auto-escaped unless you use
/// [`like()`](TextFilter::like) to supply a raw pattern.
///
/// [`deref()`](Deref::deref) exposes the inner pattern string, useful for checking
/// whether the filter has been set.
#[derive(Debug)]
pub struct TextFilter {
    col: &'static str,
    text: String,
    like: bool,
}

impl Deref for TextFilter {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

macro_rules! format_like {
    ($template: expr, $text: expr) => {
        match $text.is_empty() {
            true => String::new(),
            false => format!($template, Self::safe_escape($text)),
        }
    };
}

impl TextFilter {
    /// Create an empty filter for the given column (no match pattern — generates no WHERE clause).
    pub fn new(col: &'static str) -> Self {
        TextFilter {
            col,
            text: String::new(),
            like: false,
        }
    }

    /// Match rows where the column contains `text` (`%text%`). Special chars are auto-escaped.
    pub fn contains(&mut self, text: &str) -> &mut Self {
        self.text = format_like!("%{}%", text);
        self.like = true;
        self
    }

    /// Match rows where the column starts with `text` (`text%`). Special chars are auto-escaped.
    pub fn starts_with(&mut self, text: &str) -> &mut Self {
        self.text = format_like!("{}%", text);
        self.like = true;
        self
    }

    /// Match rows where the column ends with `text` (`%text`). Special chars are auto-escaped.
    pub fn ends_with(&mut self, text: &str) -> &mut Self {
        self.text = format_like!("%{}", text);
        self.like = true;
        self
    }

    /// Exact equality match (equivalent to `LIKE 'text'` with no wildcards).
    pub fn equals(&mut self, text: &str) -> &mut Self {
        self.text = text.to_string();
        self.like = false;
        self
    }

    /// Set the raw `LIKE` pattern directly (`%` / `_` are **not** escaped).
    /// Use this when you need manual control over wildcards.
    pub fn like(&mut self, text: &str) -> &mut Self {
        self.text = text.to_string();
        self.like = true;
        self
    }

    fn safe_escape(text: &str) -> String {
        text.replace('\\', "\\\\")
            .replace('%', "\\%")
            .replace('_', "\\_")
    }
}

impl Filter for TextFilter {
    fn apply(&self, mut sql: SqlBuilder) -> SqlBuilder {
        if self.text.is_empty() {
            return sql;
        }

        let op = match self.like {
            true => "LIKE ? ESCAPE '\\'",
            false => "= ?",
        };
        sql.where_clause.push(format!("{} {op}", self.col));
        sql.params.push(Box::new(self.text.clone()));
        sql
    }
}

/// Date range filter for `DateTime<Utc>` columns.
///
/// Supports setting an upper bound, a lower bound, or both (which collapses to an
/// equality check when the two values are equal). Unset bounds are silently ignored.
#[derive(Debug)]
pub struct DateFilter {
    col: &'static str,
    before: Option<DateTime<Utc>>,
    after: Option<DateTime<Utc>>,
}

impl DateFilter {
    /// Create an empty filter for the given column (no bounds — generates no WHERE clause).
    pub fn new(col: &'static str) -> Self {
        DateFilter {
            col,
            before: None,
            after: None,
        }
    }

    /// Set the upper bound: equivalent to `col <= date`.
    pub fn before(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.before = Some(date);
        self
    }

    /// Set the lower bound: equivalent to `col >= date`.
    pub fn after(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.after = Some(date);
        self
    }

    /// Set both bounds to the same date, collapsing to an equality check `col = date`.
    pub fn equals(&mut self, date: DateTime<Utc>) -> &mut Self {
        self.before = Some(date);
        self.after = Some(date);
        self
    }
}

impl Filter for DateFilter {
    fn apply(&self, mut sql: SqlBuilder) -> SqlBuilder {
        match (self.before, self.after) {
            (None, None) => {}
            (Some(before), None) => {
                sql.where_clause.push(format!("{} <= ?", self.col));
                sql.params.push(Box::new(before));
            }
            (None, Some(after)) => {
                sql.where_clause.push(format!("{} >= ?", self.col));
                sql.params.push(Box::new(after));
            }
            (Some(before), Some(after)) => {
                if before == after {
                    sql.where_clause.push(format!("{} = ?", self.col));
                    sql.params.push(Box::new(before));
                } else {
                    sql.where_clause
                        .push(format!("{} BETWEEN ? AND ?", self.col));
                    sql.params.push(Box::new(after));
                    sql.params.push(Box::new(before));
                }
            }
        }
        sql
    }
}

/// Exact-match filter for ID columns.
///
/// - Empty set: generates no WHERE clause.
/// - Single ID: generates `col = ?`.
/// - Multiple IDs: generates `col IN (SELECT value FROM json_each(?))` with a JSON array parameter.
///
/// [`deref()`](Deref::deref) exposes the inner `HashSet<T>`, useful for inspecting added IDs.
#[derive(Debug)]
pub struct IdFilter<T> {
    col: &'static str,
    ids: HashSet<T>,
}

impl<T> Deref for IdFilter<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

impl<T> IdFilter<T>
where
    T: Hash + Eq,
{
    /// Create an empty filter for the given column.
    pub fn new(col: &'static str) -> Self {
        IdFilter {
            col,
            ids: HashSet::new(),
        }
    }

    /// Add a single ID to the match set.
    pub fn insert(&mut self, id: T) -> &mut Self {
        self.ids.insert(id);
        self
    }

    /// Add multiple IDs to the match set.
    pub fn extend(&mut self, ids: impl IntoIterator<Item = T>) -> &mut Self {
        self.ids.extend(ids);
        self
    }
}

impl<T: Serialize + ToSql + Copy + 'static> Filter for IdFilter<T> {
    fn apply(&self, mut sql: SqlBuilder) -> SqlBuilder {
        match self.ids.len() {
            0 => {}
            1 => {
                sql.where_clause.push(format!("{} = ?", self.col));
                let id = self.ids.iter().next().unwrap();
                sql.params.push(Box::new(*id));
            }
            _ => {
                sql.where_clause
                    .push(format!("{} IN (SELECT value FROM json_each(?))", self.col));
                let json_array = serde_json::to_string(&self.ids).unwrap();
                sql.params.push(Box::new(json_array));
            }
        }
        sql
    }
}

/// All-of relational filter that works through a many-to-many join table.
///
/// Designed for filtering posts by their associated tags, authors, or collections.
///
/// - Empty set: generates no WHERE clause.
/// - Single ID: uses `EXISTS (SELECT 1 FROM <table> WHERE post = posts.id AND <col> = ?)`.
/// - Multiple IDs: ensures **all** specified IDs are present (intersection match)
///   with one `EXISTS` per ID, joined by `AND`.
///
/// [`deref()`](Deref::deref) exposes the inner `HashSet<T>`.
#[derive(Debug)]
pub struct RelationshipsFilter<T> {
    table: &'static str,
    col: &'static str,
    ids: HashSet<T>,
}

impl<T> Deref for RelationshipsFilter<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.ids
    }
}

impl<T> RelationshipsFilter<T> {
    /// Create an empty filter specifying the join table name (`table`) and the related ID column (`col`).
    pub fn new(table: &'static str, col: &'static str) -> Self {
        RelationshipsFilter {
            table,
            col,
            ids: HashSet::new(),
        }
    }
}

impl<T: Eq + Hash> RelationshipsFilter<T> {
    /// Add a single related ID to the match set.
    pub fn insert(&mut self, id: T) -> &mut Self {
        self.ids.insert(id);
        self
    }

    /// Add multiple related IDs to the match set.
    pub fn extend(&mut self, ids: impl IntoIterator<Item = T>) -> &mut Self {
        self.ids.extend(ids);
        self
    }
}

impl<T: Display + ToSql + Copy + 'static> Filter for RelationshipsFilter<T> {
    fn apply(&self, mut sql: SqlBuilder) -> SqlBuilder {
        for id in self.iter() {
            sql.where_clause.push(format!(
                "EXISTS (SELECT 1 FROM {} WHERE post = posts.id AND {} = ?)",
                self.table, self.col
            ));
            sql.params.push(Box::new(*id));
        }
        sql
    }
}
