use std::{collections::HashMap, path::PathBuf};

use serde_json::Value;

use crate::{
    manager::{binded::Binded, PostArchiverConnection},
    utils::macros::AsTable,
    FileMeta, FileMetaId, Post, PostId,
};

/// Builder for updating a file metadata's fields.
///
/// Fields left as `None` are not modified.
#[derive(Debug, Clone, Default)]
pub struct UpdateFileMeta {
    pub mime: Option<String>,
    pub extra: Option<HashMap<String, Value>>,
}

impl UpdateFileMeta {
    /// Set the MIME type.
    pub fn mime(mut self, mime: String) -> Self {
        self.mime = Some(mime);
        self
    }
    /// Set the extra metadata.
    pub fn extra(mut self, extra: HashMap<String, Value>) -> Self {
        self.extra = Some(extra);
        self
    }
}

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

    /// Apply a batch of field updates to this file metadata in a single SQL statement.
    ///
    /// Only fields set on `update` (i.e. `Some(...)`) are written to the database.
    pub fn update(&self, update: UpdateFileMeta) -> Result<(), rusqlite::Error> {
        use rusqlite::types::ToSql;

        let extra_json = update.extra.map(|e| serde_json::to_string(&e).unwrap());

        let mut sets: Vec<&str> = Vec::new();
        let mut params: Vec<&dyn ToSql> = Vec::new();

        macro_rules! push {
            ($field:expr, $col:expr) => {
                if let Some(ref v) = $field {
                    sets.push($col);
                    params.push(v);
                }
            };
        }

        push!(update.mime, "mime = ?");
        push!(extra_json, "extra = ?");

        if sets.is_empty() {
            return Ok(());
        }

        let id = self.id();
        params.push(&id);

        let sql = format!("UPDATE file_metas SET {} WHERE id = ?", sets.join(", "));
        self.conn().execute(&sql, params.as_slice())?;
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
