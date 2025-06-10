use std::hash::Hash;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
#[cfg(feature = "typescript")]
use ts_rs::TS;

use crate::{
    comment::Comment,
    id::{FileMetaId, PostId},
    Content, PlatformId,
};

/// A content entry
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
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
}
