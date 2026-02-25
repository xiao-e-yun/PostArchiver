use std::{hash::Hash, path::PathBuf};

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

impl Post {
    /// The number of posts in one chunk.
    pub const POSTS_PRE_CHUNK: u32 = 2048;

    pub fn directory(post_id: PostId) -> PathBuf {
        let id = post_id.raw();
        let chunk = id / Self::POSTS_PRE_CHUNK;
        let index = id % Self::POSTS_PRE_CHUNK;
        PathBuf::from(chunk.to_string()).join(index.to_string())
    }
}

#[cfg(feature = "utils")]
mod definitions {
    use crate::utils::macros::as_table;

    use super::*;

    as_table! {
        "posts" => Post {
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
}
