use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{CollectionId, FileMetaId, PostId};

/// A content creator or contributor in the system
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub source: Option<String>,
    pub thumb: Option<FileMetaId>,
}

impl Hash for Collection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// A label that can be applied to posts
///
/// # Safety
/// - Name must not be empty
/// - Name should be kebab-case
/// - Name can be chained (e.g. "x:y:z")
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
