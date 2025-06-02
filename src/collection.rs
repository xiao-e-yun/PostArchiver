use std::hash::Hash;

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{CollectionId, FileMetaId, PostId};

/// A content creator or contributor in the system
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Collection {
    pub id: CollectionId,
    pub name: String,
    pub description: String,
    pub thumb: Option<FileMetaId>,
}

impl Hash for Collection {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Collection {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name && self.description == other.description
    }
}

impl Eq for Collection {}

/// A label that can be applied to posts
///
/// # Safety
/// - Name must not be empty
/// - Name should be kebab-case
/// - Name can be chained (e.g. "x:y:z")
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct CollectionPost {
    pub collection: CollectionId,
    pub post: PostId,
}
