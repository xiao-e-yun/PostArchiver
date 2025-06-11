use std::collections::HashMap;

use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    FileMeta, FileMetaId, PostId,
};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Find a file's metadata by its post ID and filename.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn find_file_meta(
        &self,
        post: PostId,
        filename: &str,
    ) -> Result<Option<FileMetaId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM file_metas WHERE post = ? AND filename = ?")?;

        stmt.query_row(params![post, filename], |row| row.get(0))
            .optional()
    }
    /// Retrieve a file's metadata by its ID.
    ///
    /// Fetches all information about the file including its post ID, filename, MIME type, and extra metadata.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The file ID does not exist
    /// * There was an error accessing the database
    pub fn get_file_meta(&self, id: &FileMetaId) -> Result<FileMeta, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;
        stmt.query_row([id], FileMeta::from_row)
    }
}

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Add a new file metadata to the archive.
    ///
    /// Inserts a new file metadata with the given post ID, filename, MIME type, and extra metadata.
    /// It will check if a file with the same post and filename already exists.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The post ID does not exist
    /// * Duplicate filename for the same post
    /// * There was an error accessing the database
    pub fn add_file_meta(
        &self,
        post: PostId,
        filename: String,
        mime: String,
        extra: HashMap<String, Value>,
    ) -> Result<FileMetaId, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT INTO file_metas (post, filename, mime, extra) VALUES (?, ?, ?, ?) RETURNING id",
        )?;
        stmt.query_row(
            params![post, filename, mime, serde_json::to_string(&extra).unwrap()],
            |row| row.get(0),
        )
    }
    /// Remove a file metadata from the archive.
    ///
    /// This operation will also remove all associated thumb references.
    /// But it will not delete post.content related to this file.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_file_meta(&self, id: FileMetaId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT filename FROM file_metas WHERE id = ?")?;
        stmt.execute([id])?;
        Ok(())
    }

    /// Set the filename of a file metadata.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_file_meta_mime(&self, id: FileMetaId, mime: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_metas SET mime = ? WHERE id = ?")?;
        stmt.execute(params![mime, id])?;
        Ok(())
    }

    /// Set the extra metadata of a file metadata.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_file_meta_extra(
        &self,
        id: FileMetaId,
        extra: HashMap<String, Value>,
    ) -> Result<(), rusqlite::Error> {
        let extra_json = serde_json::to_string(&extra).unwrap();
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_metas SET extra = ? WHERE id = ?")?;
        stmt.execute(params![extra_json, id])?;
        Ok(())
    }

    // TODO: implement a method to update the filename or post of a file_meta (because these need
    // fs renames and moves)
}
