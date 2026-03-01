use std::{collections::HashMap, fs::File, hash::Hash};

use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager, UpdateFileMeta, WritableFileMeta},
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
            self.bind(id)
                .update(UpdateFileMeta::default().extra(file_meta.extra.clone()))?;
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

    /// Create or update a file metadata entry in the archive, and write `file_meta.data` to disk.
    ///
    /// Behaves like [`import_file_meta`](Self::import_file_meta) for the database entry, then
    /// writes the content of `file_meta.data` to
    /// `<archive_path>/<post_dir>/<filename>`, creating intermediate directories as needed.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_file_meta_with_content<U>(
        &self,
        post: PostId,
        file_meta: &UnsyncFileMeta<U>,
    ) -> Result<FileMetaId, rusqlite::Error>
    where
        U: WritableFileMeta,
    {
        let id = self.import_file_meta(post, file_meta)?;

        let path = self
            .path
            .join(Post::directory(post))
            .join(&file_meta.filename);

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("Failed to create directories for file content");
        }

        let mut file = File::create(&path).expect("Failed to create file for writing file content");
        file_meta
            .data
            .write_to_file(&mut file)
            .expect("Failed to write file content");

        Ok(id)
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
