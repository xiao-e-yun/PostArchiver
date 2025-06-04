use std::hash::{Hash, Hasher};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{
    comment::Comment,
    id::{FileMetaId, PostId},
    Content, PlatformId, TagId,
};

/// A content entry created by an author in the system
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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
    }
}

/// Association type that creates a many-to-many relationship between posts and tags
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct PostTag {
    pub post: PostId,
    pub tag: TagId,
}

#[cfg(feature = "utils")]
crate::utils::macros::as_table! {
    Post {
        id: "id",
        source: "source",
        title: "title",
        content: "content" => json,
        thumb: "thumb",
        comments: "comments" => json,
        updated: "updated",
        published: "published",
        platform: "platform",
    }

    PostTag {
        post: "post",
        tag: "tag",
    }
}
