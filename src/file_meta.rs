use serde::{Deserialize, Serialize};

use serde_json::Value;
#[cfg(feature = "typescript")]
use ts_rs::TS;

use std::{
    collections::HashMap,
    hash::Hash,
    ops::{Div, Rem},
    path::PathBuf,
};

use crate::id::{FileMetaId, PostId};

pub const POSTS_PRE_CHUNK: u32 = 2048;

/// Metadata for a file in the system with hierarchical path organization
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileMeta {
    pub id: FileMetaId,
    pub filename: String,
    pub post: PostId,
    pub mime: String,
    #[cfg_attr(feature = "typescript", ts(type = "Record<string, any>"))]
    pub extra: HashMap<String, Value>,
}

impl FileMeta {
    /// Returns relative path to the file.  
    /// it will be `<chunk>/<index>/<filename>`.  
    /// the `<chunk>` is `postId / POSTS_PRE_CHUNK`,
    /// the `<index>` is `postId % POSTS_PRE_CHUNK`,
    /// [`POSTS_PRE_CHUNK`] is a constant that defines how many posts are in one chunk. (default is 2048).
    ///
    /// # Examples
    /// ```rust
    /// use post_archiver::{FileMeta, AuthorId, PostId, FileMetaId};
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    ///
    /// // You should never create a FileMeta struct
    /// let file_meta = FileMeta {
    ///     id: FileMetaId::new(6),
    ///     post: PostId::new(2049),
    ///     filename: "example.txt".to_string(),
    ///     mime: "text/plain".to_string(),
    ///     extra: HashMap::new(),
    /// };
    ///
    /// let path = file_meta.path();
    /// assert_eq!(path.to_str(), Some("1/1/example.txt"));
    /// ```
    pub fn path(&self) -> PathBuf {
        let chunk = self.post.div(POSTS_PRE_CHUNK);
        let index = self.post.rem(POSTS_PRE_CHUNK);
        PathBuf::from(chunk.to_string())
            .join(index.to_string())
            .join(&self.filename)
    }
}

impl Hash for FileMeta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.post.hash(state);
        self.filename.hash(state);
    }
}

impl PartialEq for FileMeta {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.post == other.post
            && self.filename == other.filename
            && self.extra == other.extra
    }
}

impl Eq for FileMeta {}
