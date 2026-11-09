use std::{collections::HashMap, hash::Hash, path::PathBuf};

use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::{
    error::Result,
    importer::{Importer, SavepointExt},
    manager::{PostArchiverBindExt, UpdateFileMeta},
    FileMetaId, Post, PostId,
};

impl<C: SavepointExt> Importer<'_, C> {
    /// Create or update a file metadata entry in the archive.
    ///
    /// Takes a file metadata object and either creates a new entry or updates an existing one.
    /// if a file metadata with the same filename (and post id) already exists, it only updates metadata
    ///
    /// # Errors
    ///
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_file_meta(
        &mut self,
        post: PostId,
        file_meta: UnsyncFileMeta,
    ) -> Result<FileMetaId> {
        let id: Option<FileMetaId> = self.lookup_file_meta(post, &file_meta)?;

        let sp = self.conn.savepoint()?;
        let id = match id {
            Some(id) => {
                let upd = UpdateFileMeta::default().extra(file_meta.extra.clone());
                sp.bind(id).update(upd)?;
                id
            }
            None => {
                let mut ins_stmt = sp.prepare_cached(
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
                )?
            }
        };

        let path = self.path.join(Post::directory(post));
        std::fs::create_dir_all(&path)?;

        let target = path.join(&file_meta.filename);
        std::fs::rename(file_meta.source, target)?;
        sp.commit()?;

        Ok(id)
    }

    pub fn lookup_file_meta(
        &self,
        post: PostId,
        file_meta: &UnsyncFileMeta,
    ) -> Result<Option<FileMetaId>> {
        let mut stmt = self
            .conn
            .prepare_cached("SELECT id FROM file_metas WHERE post = ? AND filename = ?")?;
        let query = stmt.query_row(params![post, file_meta.filename], |row| row.get(0));
        Ok(query.optional()?)
    }
}

/// Represents a file metadata that is not yet synced to the database.
#[derive(Debug, Clone)]
pub struct UnsyncFileMeta {
    pub filename: String,
    pub mime: String,
    pub extra: HashMap<String, Value>,
    pub source: PathBuf,
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
    pub fn new(filename: String, mime: String, source: PathBuf) -> Self {
        Self {
            filename,
            mime,
            source,
            extra: HashMap::new(),
        }
    }

    pub fn extra(mut self, extra: HashMap<String, Value>) -> Self {
        self.extra = extra;
        self
    }
}
