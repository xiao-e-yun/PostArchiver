use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{id::TagId, PlatformId, PostId};

/// A label that can be applied to posts
///
/// # Safety
/// - Name must not be empty
/// - Name should be kebab-case
/// - Name can be chained (e.g. "x:y:z")
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
    pub platform: Option<PlatformId>,
}

/// Association type that creates a many-to-many relationship between posts and tags
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostTag {
    pub post: PostId,
    pub tag: TagId,
}

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Tag {
        id: "id",
        name: "name",
        platform: "platform",
    }

    PostTag {
        post: "post",
        tag: "tag",
    }
}
