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
    /// The number of post directories in one chunk directory.
    pub const POSTS_PER_CHUNK: u32 = 2048;

    /// Directory of a post's files, relative to the archive root: `<chunk>/<index>`,
    /// where `<chunk>` is `post_id / POSTS_PER_CHUNK` and `<index>` is `post_id % POSTS_PER_CHUNK`.
    ///
    /// ```
    /// use post_archiver::{Post, PostId};
    /// assert_eq!(Post::directory(PostId::new(2049)).to_str(), Some("1/1"));
    /// ```
    pub fn directory(post_id: PostId) -> PathBuf {
        let id = post_id.raw();
        let chunk = id / Self::POSTS_PER_CHUNK;
        let index = id % Self::POSTS_PER_CHUNK;
        PathBuf::from(chunk.to_string()).join(index.to_string())
    }
}

#[cfg(feature = "utils")]
mod definitions {
    use crate::{utils::macros::as_table, AuthorId, CollectionId, FileMeta, TagId};

    use super::*;

    as_table! {
        "posts" => Post(PostDefinitions) {
            column id: PostId,
            column source: Option<String>,
            column title: String,
            column content: Vec<Content> => json,
            column thumb: Option<FileMetaId>,
            column comments: Vec<Comment> => json,
            column updated: DateTime<Utc>,
            column published: DateTime<Utc>,
            column platform: Option<PlatformId>,
            relation author_posts(post => author) as AuthorId,
            relation post_tags(post => tag) as TagId,
            relation collection_posts(post => collection) as CollectionId,
            relation file_metas(post as {
                id: "id",
                mime: "mime",
                post: "post",
                filename: "filename",
                extra: "json(extra)"
            }) as FileMeta,
        }
    }
}
