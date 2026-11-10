use std::{collections::HashMap, hash::Hash, path::PathBuf};

use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::{
    error::Result,
    importer::{Importer, SavepointExt},
    manager::{PostArchiverBindExt, UpdateFileMeta},
    FileMetaId, Post, PostId,
};

impl<C: SavepointExt> Importer<'_, '_, C> {
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

        let mut sp = self.savepoint()?;
        let id = match id {
            Some(id) => {
                let upd = UpdateFileMeta::default()
                    .extra(file_meta.extra.clone())
                    .mime(file_meta.mime.clone());
                sp.conn.bind(id).update(upd)?;
                id
            }
            None => {
                let mut ins_stmt = sp.conn.prepare_cached(
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

        let path = sp.path.join(Post::directory(post));
        std::fs::create_dir_all(&path)?;

        let target = path.join(&file_meta.filename);
        sp.pending.stage(&file_meta.source, target)?;
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

/// A file to import, together with its metadata.
#[derive(Debug, Clone)]
pub struct UnsyncFileMeta {
    /// Name of the file inside the post directory. Must be a plain file name
    /// (no path separators or `..`); it is unique per post.
    pub filename: String,
    /// MIME type. The first `image/*` file of a post becomes its thumbnail when none is given.
    pub mime: String,
    /// Arbitrary metadata stored as JSON.
    pub extra: HashMap<String, Value>,
    /// File to copy into the archive. It is not moved or deleted.
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
