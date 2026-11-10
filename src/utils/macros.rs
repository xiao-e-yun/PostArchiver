use crate::query::FromQuery;

#[doc(hidden)]
pub use rusqlite;

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
            $(pub fn $field(&self, row: &$crate::utils::rusqlite::Row) -> $crate::utils::rusqlite::Result<$ty> { $crate::as_table!(@as_column, row, $field $(, $mode)?) })*
            $(pub fn $rel_table(&self, row: &$crate::utils::rusqlite::Row) -> $crate::utils::rusqlite::Result<Vec<$rty>> { $crate::as_table!(@as_column, row, $rel_table, json) })*
        }

        impl $crate::query::FromQuery for $name {
            type Based = Self;
            fn fields() -> String {
                "*".to_string()
            }
            fn from_row(row: &$crate::utils::rusqlite::Row) -> $crate::utils::rusqlite::Result<Self> {
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

/// Implement `FromQuery` for a projection of a table: a struct holding some of its columns
/// and relation columns, under the same names as in the table's `as_table!` definition.
///
/// The expansion names `rusqlite::Row`, so the calling crate needs `rusqlite` as a dependency.
///
/// ```
/// use post_archiver::{impl_from_query, manager::PostArchiverManager, query::PostArchiverQueryExt};
/// use post_archiver::{AuthorId, FileMetaId, Post, PostId};
///
/// struct PostCard {
///     id: PostId,
///     title: String,
///     thumb: Option<FileMetaId>,
///     author_posts: Vec<AuthorId>, // relation column
/// }
///
/// impl_from_query! {
///     PostCard extends Post {
///         id: id,
///         title: title,
///         thumb: thumb,
///         author_posts: author_posts,
///     }
/// }
///
/// let manager = PostArchiverManager::open_in_memory().unwrap();
/// let card: Option<PostCard> = manager.posts().get(PostId(1)).unwrap();
/// # assert!(card.is_none());
/// ```
#[macro_export]
macro_rules! impl_from_query {
    ($name:ident extends $based:ident {
        $($field:ident: $col:ident),* $(,)?
    }) => {
        impl $crate::query::FromQuery for $name {
            type Based = $based;
            fn fields() -> String {
                [ $($based::DEFINITIONS.$col),* ].join(", ")
            }
            fn from_row(row: &$crate::utils::rusqlite::Row) -> $crate::utils::rusqlite::Result<Self> {
                Ok(Self { $($field: $based::DEFINITIONS.$col(row)?,)* })
            }
        }
    };
}

pub use {as_table, impl_from_query};
