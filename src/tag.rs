use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::id::PostTagId;

/// Platform categories for tags
pub const PLATFORM_CATEGORY: &str = "platform";
/// Collection categories for tags
pub const COLLECTION_CATEGORY: &str = "collection";
/// General categories for tags
pub const GENERAL_CATEGORY: &str = "general";
/// Style categories for tags
pub const STYLE_CATEGORY: &str = "style";

/// A categorical label that can be applied to posts
///
/// # Safety
/// - (category, name) must not be empty
/// - (category, name) should be kebab-case
/// - Name can be chained (e.g. "x:y:z")
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
    pub id: PostTagId,
    pub category: String,
    pub name: String,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.category, self.name)
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.name.hash(state);
        self.category.hash(state);
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.category == other.category && self.name == other.name
    }
}

impl Eq for Tag {}
