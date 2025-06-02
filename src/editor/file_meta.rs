use std::collections::HashMap;

use rusqlite::params;
use serde_json::Value;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    FileMetaId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Set the file meta extra data by its id.
    pub fn set_file_meta_extra(
        &self,
        file_meta: FileMetaId,
        extra: &HashMap<String, Value>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_meta SET extra = ? WHERE id = ?")?;
        stmt.execute(params![serde_json::to_string(extra).unwrap(), &file_meta])?;
        Ok(())
    }
    /// Merge the file meta extra data by its id.
    pub fn merge_file_meta_extra(
        &self,
        file_meta: FileMetaId,
        extra: &HashMap<String, Value>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "UPDATE file_meta SET extra = json_patch(file_meta.extra, ?) WHERE id = ?",
        )?;
        stmt.execute(params![serde_json::to_string(extra).unwrap(), &file_meta])?;
        Ok(())
    }
    pub fn set_file_meta_mime(
        &self,
        file_meta: FileMetaId,
        mime: &str,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_meta SET mime = ? WHERE id = ?")?;
        stmt.execute(params![mime, &file_meta])?;
        Ok(())
    }
}
