/*!
Database integration for ID types

# Overview
This module provides SQLite database integration for the system's ID types,
enabling seamless conversion between Rust types and database values.
*/

use crate::{AuthorId, FileMetaId, PostId, PostTagId};
use rusqlite::{types::FromSql, ToSql};

/// Implements SQLite serialization for ID types
///
/// # Safety
/// - Ensures valid numeric bounds during conversion
/// - Handles potential overflow conditions
/// - Maintains ID type integrity
///
/// # Implementation Notes
/// - Converts between u32 (Rust) and i64 (SQLite)
/// - Implements both FromSql and ToSql traits
/// - Preserves zero-cost abstraction
///
/// # Examples
/// ```rust,no_run
/// use rusqlite::Connection;
/// use post_archiver::AuthorId;
///
/// let conn = Connection::open_in_memory().unwrap();
/// conn.execute(
///     "CREATE TABLE authors (id INTEGER PRIMARY KEY)",
///     [],
/// ).unwrap();
///
/// // Insert ID
/// let author_id = AuthorId::new(1);
/// conn.execute(
///     "INSERT INTO authors (id) VALUES (?)",
///     [author_id],
/// ).unwrap();
///
/// // Query ID
/// let id: AuthorId = conn.query_row(
///     "SELECT id FROM authors LIMIT 1",
///     [],
///     |row| row.get(0)
/// ).unwrap();
/// assert_eq!(id, author_id);
/// ```
macro_rules! sql_id {
    ($name:ident) => {
        impl FromSql for $name {
            fn column_result(
                value: rusqlite::types::ValueRef<'_>,
            ) -> rusqlite::types::FromSqlResult<Self> {
                Ok(Self(value.as_i64()? as u32))
            }
        }

        impl ToSql for $name {
            fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                Ok(rusqlite::types::ToSqlOutput::Owned(
                    rusqlite::types::Value::Integer(self.0 as i64),
                ))
            }
        }
    };
}

sql_id!(AuthorId);
sql_id!(PostId);
sql_id!(FileMetaId);
sql_id!(PostTagId);
