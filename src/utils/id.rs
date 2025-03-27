use rusqlite::{types::FromSql, ToSql};

use crate::{AuthorId, FileMetaId, PostId, PostTagId};

/// This macro defines a wrapper for the given ID type and implements the `FromSql` and `ToSql` traits
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