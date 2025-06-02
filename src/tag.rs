use std::{
    fmt::Display,
    hash::{Hash, Hasher},
};

use rusqlite::Row;
use serde::{Deserialize, Serialize};

#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{id::TagId, PlatformId, PlatformTagId, PostId};

/// A label that can be applied to posts
///
/// # Safety
/// - Name must not be empty
/// - Name should be kebab-case
/// - Name can be chained (e.g. "x:y:z")
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Tag {
    pub id: TagId,
    pub name: String,
}

impl Display for Tag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl Hash for Tag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for Tag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name
    }
}

impl Eq for Tag {}

/// Association type that creates a many-to-many relationship between posts and tags
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostTag {
    pub post: PostId,
    pub tag: TagId,
}

/// A platform classification label that can be applied to posts
///
/// # Safety
/// - Name must not be empty
/// - Name can be chained (e.g. "x:y:z")
/// - Platform must be a valid platform ID
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PlatformTag {
    pub id: PlatformTagId,
    pub platform: PlatformId,
    pub name: String,
}

impl Display for PlatformTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "platform-{}:{}", self.platform, self.name)
    }
}

impl Hash for PlatformTag {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

impl PartialEq for PlatformTag {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.name == other.name && self.platform == other.platform
    }
}

impl Eq for PlatformTag {}

/// Association type that creates a many-to-many relationship between posts and platforms-specific tags
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct PostPlatformTag {
    pub post: PostId,
    pub tag: PlatformTagId,
}
