use std::{collections::HashMap, hash::Hash};

use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    FileMetaId, PostId,
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
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_file_meta<U>(
        &self,
        post: PostId,
        file_meta: &UnsyncFileMeta<U>,
    ) -> Result<FileMetaId, rusqlite::Error> {
        // find
        let mut find_stmt = self
            .conn()
            .prepare_cached("SELECT id FROM file_metas WHERE post = ? AND filename = ?")?;
        if let Some(id) = find_stmt
            .query_row(params![post, file_meta.filename], |row| {
                row.get::<_, FileMetaId>(0)
            })
            .optional()?
        {
            // update extra
            self.bind(id).set_extra(file_meta.extra.clone())?;
            return Ok(id);
        }

        // insert
        let mut ins_stmt = self.conn().prepare_cached(
            "INSERT INTO file_metas (post, filename, mime, extra) VALUES (?, ?, ?, ?) RETURNING id",
        )?;
        ins_stmt.query_row(
            params![
                post,
                file_meta.filename,
                file_meta.mime,
                serde_json::to_string(&file_meta.extra).unwrap()
            ],
            |row| row.get(0),
        )
    }
}

/// Represents a file metadata that is not yet synced to the database.
#[derive(Debug, Clone)]
pub struct UnsyncFileMeta<T> {
    pub filename: String,
    pub mime: String,
    pub extra: HashMap<String, Value>,
    pub data: T,
}

impl<T> Hash for UnsyncFileMeta<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
        self.mime.hash(state);
    }
}

impl<T> PartialEq for UnsyncFileMeta<T> {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.mime == other.mime && self.extra == other.extra
    }
}

impl<T> Eq for UnsyncFileMeta<T> {}

impl<T> UnsyncFileMeta<T> {
    pub fn new(filename: String, mime: String, data: T) -> Self {
        Self {
            filename,
            mime,
            data,
            extra: HashMap::new(),
        }
    }

    pub fn extra(mut self, extra: HashMap<String, Value>) -> Self {
        self.extra = extra;
        self
    }
}
