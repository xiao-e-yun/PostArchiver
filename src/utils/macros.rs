use crate::query::FromQuery;

pub trait AsTable: FromQuery<Based = Self> + Sized {
    const TABLE_NAME: &'static str;
}

#[macro_export]
macro_rules! as_column {
    ($row: expr, $col: expr) => {
        $row.get($col)?
    };
    (json, $row: expr, $col: expr) => {{
        let value: String = $row.get($col)?;
        serde_json::from_str(&value).unwrap()
    }};
}

#[macro_export]
macro_rules! as_table {
    ($($table:expr => $name:ident { $($field:ident: $col:expr $(=> $mode: ident)?),* $(,)? })+) => {
        $(impl $crate::utils::macros::AsTable for $name {
            const TABLE_NAME: &'static str = $table;
        }

        impl $crate::query::FromQuery for $name {
            type Based = Self;
            fn select_sql() -> String {
                format!("SELECT * FROM {}", <Self as crate::utils::macros::AsTable>::TABLE_NAME)
            }
            fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self {
                    $(
                        $field: crate::utils::macros::as_column!(
                            $($mode ,)?
                            row,
                            $col
                        ),
                    )*
                })
            }
        })+
    };
}

/// Helper macro to implement `FromQuery` for a struct with custom column mappings and optional JSON deserialization.
/// ```
/// # use post_archiver::{utils::macros::impl_from_query, *};
///
/// struct PostPreview {
///     id: i64,
///     title: String,
///     thumb: Option<FileMetaId>,
///     updated: i64,
///     comments: Vec<Comment>,
/// }
///
/// impl_from_query! {
///     PostPreview extends Post {
///         id: "id",
///         title: "title",
///         thumb: "thumb",
///         updated: "updated",
///         comments: "comments" => json,
///     }
/// }
/// ```
#[macro_export]
macro_rules! impl_from_query {
    ($name:ident extends $based:ty { $($field:ident: $col:expr $(=> $mode: ident)?),* $(,)? }) => {
        impl $crate::query::FromQuery for $name {
            type Based = $based;
            fn select_sql() -> String {
                format!("SELECT {:?} FROM {}", [$($col),*].join(","), <$based as $crate::utils::macros::AsTable>::TABLE_NAME)
            }
            fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self {
                    $(
                        $field: $crate::utils::macros::as_column!(
                            $($mode ,)?
                            row,
                            $col
                        ),
                    )*
                })
            }
        }
    };
}

pub use {as_column, as_table, impl_from_query};
