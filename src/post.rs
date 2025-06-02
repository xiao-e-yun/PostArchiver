use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{
    comment::Comment,
    id::{FileMetaId, PostId},
    Content, PlatformId,
};

/// A content entry created by an author in the system
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Post {
    pub id: PostId,
    pub source: Option<String>,
    pub title: String,
    pub content: Vec<Content>,
    pub thumb: Option<FileMetaId>,
    pub comments: Vec<Comment>,
    pub updated: DateTime<Utc>,
    pub published: DateTime<Utc>,
    pub platform: Option<PlatformId>,
}

impl Hash for Post {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.source.hash(state);
        // update will not change the hash
        self.published.hash(state);
        self.platform.hash(state);
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.source == other.source
            && self.updated == other.updated
            && self.published == other.published
            && self.platform == other.platform
    }
}
impl Eq for Post {}
