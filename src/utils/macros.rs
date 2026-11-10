use crate::query::FromQuery;

pub trait AsTable: FromQuery<Based = Self> + Sized {
    const TABLE_NAME: &'static str;
}

#[macro_export]
macro_rules! as_table {
    // private helper
    (@as_column, $row: expr, $col: expr) => {
        $row.get(stringify!($col))
    };
    (@as_column, $row: expr, $col: expr, json) => {
        $crate::utils::macros::json_column($row, stringify!($col))
    };
    (@sub_query, $table: literal, $rel_table: ident, $rel_field: ident as { $($rests: tt)* }) => {
        concat!( "(SELECT json_group_array(json_object(", $crate::as_table!(@sql_params, $($rests)*) ,")) FROM ", stringify!($rel_table), " WHERE ", stringify!($rel_field), " = ", $table, ".id) AS ", stringify!($rel_table))
    };
    (@sub_query, $table: literal, $rel_table: ident, $rel_field: ident => $rel_value: tt) => {
        concat!( "(SELECT json_group_array(", stringify!($rel_value), ") FROM ", stringify!($rel_table), " WHERE ", stringify!($rel_field), " = ", $table, ".id) AS ", stringify!($rel_table))
    };
    (@sql_params, $key: ident: $value: literal, $($rests:tt)*) => {
        concat!("'", stringify!($key), "', ", $value, ", ", $crate::as_table!(@sql_params, $($rests)*))
    };
    (@sql_params, $key: ident: $value: literal) => {
        concat!("'", stringify!($key), "', ", $value)
    };
    // public interface
    ($($table:literal => $name:ident ($definitions: ident) {
        // id: PostId[ => json]
        $(column $field: ident: $ty: ty $(=> $mode: ident)? ,)*
        // author_posts(post[ => author])
        $(relation $rel_table: ident ($rel_field: ident $($rests: tt)*) as $rty:ty ,)*
    })+) => {
        $(impl $crate::utils::macros::AsTable for $name {
            const TABLE_NAME: &'static str = $table;
        }

        impl $name {
            #[doc(hidden)]
            pub const DEFINITIONS: $definitions = $definitions {
                $($field: stringify!($field),)*
                $($rel_table: $crate::as_table!(@sub_query, $table, $rel_table, $rel_field $($rests)*),)*
            };
        }

        #[doc(hidden)]
        pub struct $definitions {
            $(pub $field: &'static str,)*
            $(pub $rel_table: &'static str,)*
        }
        impl $definitions {
            $(pub fn $field(&self, row: &rusqlite::Row) -> Result<$ty, rusqlite::Error> { $crate::as_table!(@as_column, row, $field $(, $mode)?) })*
            $(pub fn $rel_table(&self, row: &rusqlite::Row) -> Result<Vec<$rty>, rusqlite::Error> { $crate::as_table!(@as_column, row, $rel_table, json) })*
        }

        impl $crate::query::FromQuery for $name {
            type Based = Self;
            fn fields() -> String {
                "*".to_string()
            }
            fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self {
                    $( $field: $name::DEFINITIONS.$field(row)?, )*
                })
            }
        })+
    };
}

#[doc(hidden)]
pub fn json_column<T: serde::de::DeserializeOwned>(
    row: &rusqlite::Row,
    col: &str,
) -> Result<T, rusqlite::Error> {
    let value: String = row.get(col)?;
    serde_json::from_str(&value).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(e))
    })
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
    ($name:ident extends $based:ident {
        $($field:ident: $col:ident),* $(,)?
    }) => {
        impl $crate::query::FromQuery for $name {
            type Based = $based;
            fn fields() -> String {
                [ $($based::DEFINITIONS.$field),* ].join(", ")
            }
            fn from_row(row: &rusqlite::Row) -> Result<Self, rusqlite::Error> {
                Ok(Self { $($field: $based::DEFINITIONS.$col(row)?,)* })
            }
        }
    };
}

pub use {as_table, impl_from_query};
