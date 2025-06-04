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
    /// Look up file metadata by post ID and filename, returning its ID if found.
    ///
    /// Given a post ID and filename, searches the database for a matching file metadata entry.
    /// Returns `Some(FileMetaId)` if found, `None` if no matching metadata exists.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error querying the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::PostId;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let post_id = PostId(1);
    ///     
    ///     if let Some(id) = manager.check_file_meta(post_id, "image.jpg")? {
    ///         println!("File metadata exists with ID: {}", id);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
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
    /// Retrieve an file's complete information from the archive.
    ///
    /// Fetches all information about a post including its content, comments, and metadata.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The fileMeta ID does not exist
    /// * There was an error accessing the database
    /// * The stored data is malformed
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::FileMetaId;
    /// fn example(manager: &PostArchiverManager, id: FileMetaId) -> Result<(), Box<dyn std::error::Error>> {
    ///     let file_meta = manager.get_file_meta(&id)?;
    ///     println!("file name: {}", file_meta.filename);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn get_file_meta(&self, id: &FileMetaId) -> Result<FileMeta, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;
        stmt.query_row([id], |row| FileMeta::from_row(row))
    }
}

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
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
    pub fn remove_file_meta(&self, id: FileMetaId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT filename FROM file_metas WHERE id = ?")?;
        stmt.execute([id])?;
        Ok(())
    }

    pub fn set_file_meta_mime(&self, id: FileMetaId, mime: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_metas SET mime = ? WHERE id = ?")?;
        stmt.execute(params![mime, id])?;
        Ok(())
    }

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
