use serde::{Deserialize, Serialize};

use serde_json::Value;
#[cfg(feature = "typescript")]
use ts_rs::TS;

use std::{collections::HashMap, hash::Hash, path::PathBuf};

use crate::id::{AuthorId, FileMetaId, PostId};

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
    /// Returns the path of the file  
    /// example: `author/post/filename`
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
