use std::{collections::HashMap, hash::Hash};

use rusqlite::params;
use serde_json::Value;

use crate::{
    error::Result,
    manager::{
        file_meta::FileMetaContent, PostArchiverConnection, PostArchiverManager, UpdateFileMeta,
    },
    FileMetaId, Post, PostId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Create or update a file metadata entry in the archive.
    ///
    /// Takes a file metadata object and either creates a new entry or updates an existing one.
    /// if a file metadata with the same filename (and post id) already exists, it only updates metadata
    ///
    /// # Errors
    ///
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_file_meta(&self, post: PostId, file_meta: &UnsyncFileMeta) -> Result<FileMetaId> {
        // find
        if let Some(id) = self.find_file_meta(post, &file_meta.filename)? {
            // update extra
            self.bind(id)
                .update(UpdateFileMeta::default().extra(file_meta.extra.clone()))?;
            return Ok(id);
        }

        let path = self.path.join(Post::directory(post));
        if !path.exists() {
            std::fs::create_dir_all(&path)?;
        }
        file_meta.data.write(&path.join(&file_meta.filename))?;

        // insert
        let mut ins_stmt = self.conn().prepare_cached(
            "INSERT INTO file_metas (post, filename, mime, extra) VALUES (?, ?, ?, ?) RETURNING id",
        )?;
        Ok(ins_stmt.query_row(
            params![
                post,
                file_meta.filename,
                file_meta.mime,
                serde_json::to_string(&file_meta.extra).unwrap()
            ],
            |row| row.get(0),
        )?)
    }
}

/// Represents a file metadata that is not yet synced to the database.
#[derive(Debug, Clone)]
pub struct UnsyncFileMeta {
    pub filename: String,
    pub mime: String,
    pub extra: HashMap<String, Value>,
    pub data: FileMetaContent,
}

impl Hash for UnsyncFileMeta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
        self.mime.hash(state);
    }
}

impl PartialEq for UnsyncFileMeta {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.mime == other.mime && self.extra == other.extra
    }
}

impl Eq for UnsyncFileMeta {}

impl UnsyncFileMeta {
    pub fn new(filename: String, mime: String, data: FileMetaContent) -> Self {
        Self {
            filename,
            mime,
            data: data.into(),
            extra: HashMap::new(),
        }
    }

    pub fn extra(mut self, extra: HashMap<String, Value>) -> Self {
        self.extra = extra;
        self
    }
}
