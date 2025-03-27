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

/// Represents a post
/// 
/// # Structure
/// `id` Unique identifier for the post  
/// `author` Author of the post  
/// `source` Optional source reference for the post  
/// `title` Title of the post  
/// `content` Content of the post, which can be either text or a file reference  
/// `thumb` Optional thumbnail/avatar image reference  
/// `comments` Collection of comments associated with the post  
/// `updated` Timestamp of when the post was last updated  
/// `published` Timestamp of when the post was published  
/// 
/// # Relationships
/// [`Author`](crate::author::Author) - Represents the author of the post  
/// [`PostTag`](crate::post::tag::PostTag) - Represents the association between a post and its tags
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
