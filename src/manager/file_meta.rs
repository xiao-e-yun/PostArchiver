use std::{collections::HashMap, fs::File, io::Write, path::PathBuf};

use serde_json::Value;

use crate::{
    manager::{binded::Binded, PostArchiverConnection},
    utils::macros::AsTable,
    FileMeta, FileMetaId, Post, PostId,
};

/// Builder for updating a file metadata's fields.
///
/// Fields left as `None` are not modified.
#[derive(Debug, Clone)]
pub struct UpdateFileMeta<T> {
    pub mime: Option<String>,
    pub extra: Option<HashMap<String, Value>>,
    pub content: Option<T>,
}

impl Default for UpdateFileMeta<()> {
    fn default() -> Self {
        UpdateFileMeta {
            mime: None,
            extra: None,
            content: None,
        }
    }
}

impl<T> UpdateFileMeta<T> {
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
    pub fn content<U: WritableFileMeta>(self, content: U) -> UpdateFileMeta<U> {
        UpdateFileMeta {
            content: Some(content),
            mime: self.mime,
            extra: self.extra,
        }
    }
}

impl UpdateFileMeta<()> {
    /// Convert this update to a version without content, for use in the `update` method.
    pub fn new() -> UpdateFileMeta<()> {
        UpdateFileMeta {
            content: None,
            mime: None,
            extra: None,
        }
    }
}

pub trait WritableFileMeta {
    fn write_to_file(&self, file: &mut File) -> std::io::Result<()>;
}

macro_rules! can_be_content {
    ($t:ty) => {
        impl UpdateFileMeta<$t> {
            pub fn new(content: $t) -> Self {
                Self {
                    content: Some(content),
                    mime: None,
                    extra: None,
                }
            }
        }
    };
}

can_be_content!(File);
impl WritableFileMeta for File {
    fn write_to_file(&self, file: &mut File) -> std::io::Result<()> {
        let mut src_file = self.try_clone()?;
        std::io::copy(&mut src_file, file)?;
        file.sync_data()?;
        Ok(())
    }
}

can_be_content!(Vec<u8>);
impl WritableFileMeta for Vec<u8> {
    fn write_to_file(&self, file: &mut File) -> std::io::Result<()> {
        file.write_all(self)?;
        file.sync_data()?;
        Ok(())
    }
}

can_be_content!(PathBuf);
impl WritableFileMeta for PathBuf {
    fn write_to_file(&self, file: &mut File) -> std::io::Result<()> {
        let mut src_file = File::open(self)?;
        std::io::copy(&mut src_file, file)?;
        file.sync_data()?;
        Ok(())
    }
}

can_be_content!(String);
impl WritableFileMeta for String {
    fn write_to_file(&self, file: &mut File) -> std::io::Result<()> {
        file.write_all(self.as_bytes())?;
        file.sync_data()?;
        Ok(())
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
    pub fn update<T>(&self, update: UpdateFileMeta<T>) -> Result<(), rusqlite::Error> {
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

        let sql = format!("UPDATE file_metas SET {} WHERE id = ?", sets.join(", "));
        let id = self.id();
        params.push(&id);
        self.conn().execute(&sql, params.as_slice())?;

        Ok(())
    }

    pub fn update_with_content<T>(
        &self,
        mut update: UpdateFileMeta<T>,
    ) -> Result<(), rusqlite::Error>
    where
        T: WritableFileMeta,
    {
        let content = update.content.take();
        self.update(UpdateFileMeta {
            content: None,
            ..update
        })?;

        let path = self.get_path()?;

        if let Some(content) = content {
            let mut file =
                File::create(path).expect("Failed to create file for writing file content");
            content
                .write_to_file(&mut file)
                .expect("Failed to write file content");
        }

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
