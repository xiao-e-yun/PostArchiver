use serde::{Deserialize, Serialize};

use serde_json::Value;
#[cfg(feature = "typescript")]
use ts_rs::TS;

use std::{collections::HashMap, hash::Hash, path::PathBuf};

use crate::id::{AuthorId, FileMetaId, PostId};

/// Represents a file metadata
/// 
/// # Structure
/// `id` id of the file
/// `filename` Name of the file  
/// `author` Author of the file  
/// `post` Post associated with the file  
/// `mime` MIME type of the file  
/// `extra` Additional metadata associated with the file  
///
/// # Relationships
/// [`Author`](crate::author::Author) - Represents the author of the file  
/// [`Post`](crate::post::Post) - Represents a post that the file belongs to  
#[cfg_attr(feature = "typescript", derive(TS))]
#[cfg_attr(feature = "typescript", ts(export))]
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct FileMeta {
    pub id: FileMetaId,
    pub filename: String,
    pub author: AuthorId,
    pub post: PostId,
    pub mime: String,
    #[cfg_attr(feature = "typescript", ts(type = "Record<string, any>"))]
    pub extra: HashMap<String,Value>
}

impl FileMeta {
    /// Returns relative path to the file.  
    /// it will be `<author>/<post>/<filename>`.  
    /// 
    /// # Example
    /// ```rust
    /// use post_archiver::{FileMeta, AuthorId, PostId, FileMetaId};
    /// use std::collections::HashMap;
    /// use std::path::PathBuf;
    /// 
    /// let file_meta = FileMeta {
    ///     id: FileMetaId::new(6),
    ///     author: AuthorId::new(1),
    ///     post: PostId::new(2),
    ///     filename: "example.txt".to_string(),
    ///     mime: "text/plain".to_string(),
    ///     extra: HashMap::new(),
    /// };
    /// 
    /// let path = file_meta.path();
    /// assert_eq!(path.to_str(), Some("1/2/example.txt"));
    /// ```
    /// 
    pub fn path(&self) -> PathBuf {
        PathBuf::from(self.author.to_string())
            .join(self.post.to_string())
            .join(&self.filename)
    }
}

impl Hash for FileMeta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id.hash(state);
        self.post.hash(state);
        self.author.hash(state);
        self.mime.hash(state);
        self.filename.hash(state);
    }
}

impl PartialEq for FileMeta {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.post == other.post
            && self.author == other.author
            && self.filename == other.filename
            && self.mime == other.mime
            && self.extra == other.extra
    }
}

impl Eq for FileMeta {}
