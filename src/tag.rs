use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::PostTagId;

/// A categorical label that can be applied to posts
///
/// # Safety
/// - Tag names must not be empty
/// - Tag names should be kebab-case
/// - IDs must be unique across all tags
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
    pub id: PostTagId,
    pub name: String,
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Eq for Tag {}
