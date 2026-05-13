use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
};

use serde_json::Value;

use crate::{
    error::Result,
    manager::{binded::Binded, PostArchiverConnection},
    query::FromQuery,
    FileMeta, FileMetaId, Post, PostId,
};

/// Builder for updating a file metadata's fields.
///
/// Fields left as `None` are not modified.
#[derive(Debug, Clone)]
pub struct UpdateFileMeta {
    pub mime: Option<String>,
    pub extra: Option<HashMap<String, Value>>,
    pub content: Option<FileMetaContent>,
}

impl Default for UpdateFileMeta {
    fn default() -> Self {
        UpdateFileMeta {
            mime: None,
            extra: None,
            content: None,
        }
    }
}

impl UpdateFileMeta {
    /// Convert this update to a version, for use in the `update` method.
    pub fn new() -> UpdateFileMeta {
        UpdateFileMeta {
            content: None,
            mime: None,
            extra: None,
        }
    }

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
    /// Set the file content by path.
    pub fn path(mut self, content: impl AsRef<Path>) -> UpdateFileMeta {
        self.content = Some(FileMetaContent::path(content));
        self
    }

    /// Set the file content by bytes.
    pub fn content(mut self, content: Vec<u8>) -> Self {
        self.content = Some(FileMetaContent::content(content));
        self
    }
}

//=============================================================
// Update / Delete
//=============================================================
impl<'a, C: PostArchiverConnection> Binded<'a, FileMetaId, C> {
    /// Get this file metadata's current data from the database.
    pub fn value(&self) -> Result<FileMeta> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;
        Ok(stmt.query_row([self.id()], FileMeta::from_row)?)
    }

    /// Remove this file metadata from the archive.
    ///
    /// This operation will also remove all associated thumb references.
    /// But it will not delete post.content related to this file.
    pub fn delete(self) -> Result<()> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM file_metas WHERE id = ?")?;
        stmt.execute([self.id()])?;
        Ok(())
    }

    /// Apply a batch of field updates to this file metadata in a single SQL statement.
    ///
    /// Only fields set on `update` (i.e. `Some(...)`) are written to the database.
    pub fn update(&self, update: UpdateFileMeta) -> Result<()> {
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

        let sql = format!("UPDATE file_metas SET {} WHERE id = ?", sets.join(", "));
        let id = self.id();
        params.push(&id);
        self.conn().execute(&sql, params.as_slice())?;

        Ok(())
    }

    /// Get the file path of this file metadata.
    pub fn get_path(&self) -> Result<PathBuf> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT post, filename FROM file_metas WHERE id = ?")?;
        Ok(stmt.query_row([self.id()], |row| {
            let post_id: PostId = row.get(0)?;
            let filename: String = row.get(1)?;
            Ok(Post::directory(post_id).join(filename))
        })?)
    }
}

//=============================================================
// File content
//=============================================================

#[derive(Debug, Clone, PartialEq)]
pub enum FileMetaContent {
    Content(Vec<u8>),
    Path(PathBuf),
}

impl FileMetaContent {
    pub fn content(content: Vec<u8>) -> Self {
        FileMetaContent::Content(content)
    }
    pub fn path(path: impl AsRef<Path>) -> Self {
        FileMetaContent::Path(path.as_ref().to_path_buf())
    }
    pub fn write(&self, path: &Path) -> Result<()> {
        match self {
            FileMetaContent::Content(content) => {
                let mut file = File::create(path)?;
                file.write_all(content)?;
            }
            FileMetaContent::Path(src_path) => {
                fs::copy(src_path, path)?;
            }
        };
        Ok(())
    }
}
