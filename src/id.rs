use crate::utils::macros::wrraper;

use core::fmt;
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

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
    ($(#[$meta:meta])*,$name:ident) => {
        #[cfg_attr(feature = "typescript", derive(TS))]
        #[cfg_attr(feature = "typescript", ts(export))]
        #[derive(
            Deserialize,
            Serialize,
            Debug,
            Clone,
            Copy,
            Hash,
            PartialEq,
            Eq,
        )]
        pub struct $name(pub u32);
        wrraper!($name: u32);

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
    };
}

define_id!(
/// Unique identifier for an author in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,AuthorId);

define_id!(
/// Unique identifier for a post in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,PostId);

define_id!(
/// Unique identifier for a file metadata entry in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,FileMetaId);

define_id!(
/// Unique identifier for a post tag in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,TagId);

define_id!(
/// Unique identifier for a post tag that is platform-specific
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,PlatformTagId);

define_id!(
/// Unique identifier for a platform in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,PlatformId);

define_id!(
/// Unique identifier for a collection in the system
///
/// # Safety
/// - The wrapped value must be a valid u32
/// - Must maintain referential integrity when used as a foreign key
,CollectionId);
