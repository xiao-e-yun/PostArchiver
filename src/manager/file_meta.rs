use std::{collections::HashMap, path::PathBuf};

use rusqlite::params;
use serde_json::Value;

use crate::{
    manager::{binded::Binded, PostArchiverConnection},
    utils::macros::AsTable,
    FileMeta, FileMetaId, Post, PostId,
};

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, FileMetaId, C> {
    /// Get this file metadata's current data from the database.
    pub fn value(&self) -> Result<FileMeta, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;
        stmt.query_row([self.id()], FileMeta::from_row)
    }

    /// Remove this file metadata from the archive.
    ///
    /// This operation will also remove all associated thumb references.
    /// But it will not delete post.content related to this file.
    pub fn delete(self) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM file_metas WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Set the MIME type of this file metadata.
    pub fn set_mime(&self, mime: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_metas SET mime = ? WHERE id = ?")?;
        stmt.execute(params![mime, self.id()])?;
        Ok(())
    }

    /// Set the extra metadata of this file metadata.
    pub fn set_extra(&self, extra: HashMap<String, Value>) -> Result<(), rusqlite::Error> {
        let extra_json = serde_json::to_string(&extra).unwrap();
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_metas SET extra = ? WHERE id = ?")?;
        stmt.execute(params![extra_json, self.id()])?;
        Ok(())
    }

    /// Get the file path of this file metadata.
    pub fn get_path(&self) -> Result<PathBuf, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post, filename FROM file_metas WHERE id = ?")?;
        stmt.query_row([self.id()], |row| {
            let post_id: PostId = row.get(0)?;
            let filename: String = row.get(1)?;
            Ok(Post::directory(post_id).join(filename))
        })
    }
}
