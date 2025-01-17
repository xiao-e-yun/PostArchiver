use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

pub mod content;
pub mod tag;

pub use content::*;
pub use tag::*;

use crate::{
    comment::Comment,
    id::{AuthorId, FileMetaId, PostId},
};

#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Post {
    pub id: PostId,
    pub author: AuthorId,
    pub source: Option<String>,
    pub title: String,
    pub content: Vec<Content>,
    pub thumb: Option<FileMetaId>,
    pub comments: Vec<Comment>,
    pub updated: DateTime<Utc>,
    pub published: DateTime<Utc>,
}

impl Hash for Post {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.author.hash(state);
        self.source.hash(state);
        // update will not change the hash
        self.published.hash(state);
    }
}

impl PartialEq for Post {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.author == other.author
            && self.source == other.source
            && self.updated == other.updated
            && self.published == other.published
    }
}
impl Eq for Post {}
