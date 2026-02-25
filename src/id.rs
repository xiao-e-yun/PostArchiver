use core::fmt;
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{Author, Collection, FileMeta, Platform, Post, Tag};

pub trait HasId {
    type Id: std::hash::Hash + Eq + Clone;
    fn id(&self) -> Self::Id;
}

/// Defines a strongly-typed numeric identifier type
///
/// # Safety
/// - The value must never be negative
/// - The maximum value is constrained by u32::MAX
///
/// # Examples
/// ```rust
/// use post_archiver::{AuthorId, PostId};
///
/// // Create an author ID
/// let author_id = AuthorId::new(1);
/// assert_eq!(author_id.raw(), 1);
///
/// // Convert from usize
/// let id_from_usize = AuthorId::from(2_usize);
/// assert_eq!(id_from_usize.to_string(), "2");
///
/// // Type safety demonstration
/// let post_id = PostId::new(1);
///
/// // This will not compile:
/// // let _: PostId = author_id;
/// ```
macro_rules! define_id {
    ($(#[$meta:meta])*,$table:ty : $name:ident) => {
        #[cfg_attr(feature = "typescript", derive(TS))]
        #[cfg_attr(feature = "typescript", ts(export))]
        #[derive(Deserialize, Serialize, Debug, Clone, Copy, Hash, PartialEq, Eq)]
        pub struct $name(pub u32);

        impl core::ops::Deref for $name {
            type Target = u32;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl core::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<u32> for $name {
            fn from(f: u32) -> Self {
                Self(f)
            }
        }

        impl From<$name> for u32 {
            fn from(t: $name) -> Self {
                t.0
            }
        }

        impl $name {
            pub fn new(id: u32) -> Self {
                Self(id)
            }
            /// get the raw value of the id
            pub fn raw(&self) -> u32 {
                self.0
            }
        }

        impl From<usize> for $name {
            fn from(id: usize) -> Self {
                Self(id as u32)
            }
        }

        impl From<$name> for usize {
            fn from(id: $name) -> usize {
                id.0 as usize
            }
        }

        impl AsRef<u32> for $name {
            fn as_ref(&self) -> &u32 {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        #[cfg(feature = "utils")]
        impl rusqlite::types::FromSql for $name {
            fn column_result(
                value: rusqlite::types::ValueRef<'_>,
            ) -> rusqlite::types::FromSqlResult<Self> {
                Ok(Self(value.as_i64()? as u32))
            }
        }

        #[cfg(feature = "utils")]
        impl rusqlite::types::ToSql for $name {
            fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
                Ok(rusqlite::types::ToSqlOutput::Owned(
                    rusqlite::types::Value::Integer(self.0 as i64),
                ))
            }
        }

        #[cfg(feature = "utils")]
        impl crate::id::HasId for $table {
            type Id = $name;
            fn id(&self) -> $name {
                self.id.clone()
            }
        }
    };
}

define_id!(
/// Unique identifier for an author in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,Author: AuthorId);

define_id!(
/// Unique identifier for a post in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,Post: PostId);

define_id!(
/// Unique identifier for a file metadata entry in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,FileMeta: FileMetaId);

define_id!(
/// Unique identifier for a post tag in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,Tag: TagId);

define_id!(
/// Unique identifier for a platform in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,Platform: PlatformId);

define_id!(
/// Unique identifier for a collection in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,Collection: CollectionId);
