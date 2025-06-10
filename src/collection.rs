use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{CollectionId, FileMetaId, PostId};

/// A content creator or contributor in the system
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub source: Option<String>,
    pub thumb: Option<FileMetaId>,
}

/// Association type that creates a many-to-many relationship between collections and posts
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct CollectionPost {
    pub collection: CollectionId,
    pub post: PostId,
}

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Collection {
        id: "id",
        name: "name",
        thumb: "thumb",
        source: "source",
    }

    CollectionPost {
        collection: "collection",
        post: "post",
    }
}
