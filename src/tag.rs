use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::PostTagId;

/// Represents a tag for a post
///
/// # Structure
/// `id` Unique identifier for the tag  
/// `name` Name of the tag  
/// 
/// # Relationships
/// [`PostTag`](crate::post::tag::PostTag) - Represents the association between a post and its tags
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
