use crate::query::FromQuery;

pub trait AsTable: FromQuery<Based = Self> + Sized {
    const TABLE_NAME: &'static str;
}

macro_rules! as_table {
    // private helper
    (@as_column, $row: expr, $col: expr) => {
        $row.get(stringify!($col))
    };
    (@as_column, $row: expr, $col: expr, json) => {
        $crate::utils::macros::json_column($row, stringify!($col))
    };
    (@sub_query, $table: literal, $rel_table: ident, $rel_field: ident as { $($rests: tt)* }) => {
        concat!( "(SELECT json_group_array(json_object(", $crate::utils::macros::as_table!(@sql_params, $($rests)*) ,")) FROM ", stringify!($rel_table), " WHERE ", stringify!($rel_field), " = ", $table, ".id) AS ", stringify!($rel_table))
    };
    (@sub_query, $table: literal, $rel_table: ident, $rel_field: ident => $rel_value: tt) => {
        concat!( "(SELECT json_group_array(", stringify!($rel_value), ") FROM ", stringify!($rel_table), " WHERE ", stringify!($rel_field), " = ", $table, ".id) AS ", stringify!($rel_table))
    };
    (@sql_params, $key: ident: $value: literal, $($rests:tt)*) => {
        concat!("'", stringify!($key), "', ", $value, ", ", $crate::utils::macros::as_table!(@sql_params, $($rests)*))
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
                $($rel_table: $crate::utils::macros::as_table!(@sub_query, $table, $rel_table, $rel_field $($rests)*),)*
            };
        }

        #[doc(hidden)]
        pub struct $definitions {
            $(pub $field: &'static str,)*
            $(pub $rel_table: &'static str,)*
        }
        impl $definitions {
            $(pub fn $field(&self, row: &rusqlite::Row) -> rusqlite::Result<$ty> { $crate::utils::macros::as_table!(@as_column, row, $field $(, $mode)?) })*
            $(pub fn $rel_table(&self, row: &rusqlite::Row) -> rusqlite::Result<Vec<$rty>> { $crate::utils::macros::as_table!(@as_column, row, $rel_table, json) })*
        }

        impl $crate::query::FromQuery for $name {
            type Based = Self;
            fn fields() -> String {
                "*".to_string()
            }
            fn from_row(row: &rusqlite::Row) -> rusqlite::Result<Self> {
                Ok(Self {
                    $( $field: $name::DEFINITIONS.$field(row)?, )*
                })
            }
        })+
    };
}

pub(crate) fn json_column<T: serde::de::DeserializeOwned>(
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
/// Each mapping is `struct_field: column`, so a field may be named differently from its column.
///
/// # Relation columns
///
/// Besides its own columns, a projection can read related rows in the same query (each one is a
/// correlated subquery, so a row with no relations gives an empty `Vec`):
///
/// | Table | Relation column | Type |
/// |-------|-----------------|------|
/// | [`Post`](crate::Post) | `author_posts` | `Vec<AuthorId>` |
/// | | `post_tags` | `Vec<TagId>` |
/// | | `collection_posts` | `Vec<CollectionId>` |
/// | | `file_metas` | `Vec<FileMeta>` (whole rows) |
/// | [`Author`](crate::Author) | `author_posts` | `Vec<PostId>` |
/// | | `author_aliases` | `Vec<Alias>` (whole rows) |
/// | [`Tag`](crate::Tag) | `post_tags` | `Vec<PostId>` |
/// | [`Collection`](crate::Collection) | `collection_posts` | `Vec<PostId>` |
/// | [`Platform`](crate::Platform) | `posts` | `Vec<PostId>` |
/// | | `tags` | `Vec<TagId>` |
/// | | `author_aliases` | `Vec<AuthorId>` |
///
/// They work with [`Repo::get`](crate::query::Repo::get),
/// [`Repo::get_many`](crate::query::Repo::get_many) and [`Query::query`](crate::query::Query::query).
/// In a list every row runs the subqueries, so keep list projections to what you show.
///
/// # Examples
///
/// A post card with its author ids:
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
///
/// A post page and an author page, each read in one query (whole `FileMeta` / `Alias` rows):
///
/// ```
/// use post_archiver::{impl_from_query, manager::PostArchiverManager, query::PostArchiverQueryExt};
/// use post_archiver::{Alias, Author, AuthorId, Content, FileMeta, Post, PostId, TagId};
///
/// struct PostPage {
///     title: String,
///     content: Vec<Content>,
///     tags: Vec<TagId>,          // ids: load the rows with `manager.tags().get_many(tags)`
///     files: Vec<FileMeta>,      // every file of the post
/// }
///
/// impl_from_query! {
///     PostPage extends Post {
///         title: title,
///         content: content,
///         tags: post_tags,
///         files: file_metas,
///     }
/// }
///
/// struct AuthorPage {
///     name: String,
///     aliases: Vec<Alias>,
/// }
///
/// impl_from_query! {
///     AuthorPage extends Author {
///         name: name,
///         aliases: author_aliases,
///     }
/// }
///
/// let manager = PostArchiverManager::open_in_memory().unwrap();
/// # manager.conn.execute_batch(r#"
/// #     INSERT INTO posts (id, title, content) VALUES (1, 'Hello', '["hi", 1]');
/// #     INSERT INTO file_metas (id, post, filename, mime) VALUES (1, 1, 'a.png', 'image/png');
/// #     INSERT INTO authors (id, name) VALUES (1, 'Alice');
/// #     INSERT INTO author_aliases (source, platform, target) VALUES ('alice', 0, 1);
/// # "#).unwrap();
/// let post: PostPage = manager.posts().get(PostId(1)).unwrap().unwrap();
/// assert_eq!(post.title, "Hello");
/// assert!(post.tags.is_empty());
/// assert_eq!(post.files[0].filename, "a.png");
///
/// let author: AuthorPage = manager.authors().get(AuthorId(1)).unwrap().unwrap();
/// assert_eq!((author.name.as_str(), author.aliases[0].source.as_str()), ("Alice", "alice"));
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

pub(crate) use as_table;
pub use impl_from_query;
