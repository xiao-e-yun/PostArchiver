use std::{collections::HashMap, hash::Hash, path::PathBuf};

use rusqlite::{params, OptionalExtension};

use crate::{AuthorId, FileMeta, FileMetaId, PostId};

use super::{ImportConnection, PostArchiverImporter};

impl<T> PostArchiverImporter<T>
where
    T: ImportConnection,
{
    pub fn check_file_meta(
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
    pub fn import_file_meta(
        &self,
        author: AuthorId,
        post: PostId,
        file_meta: UnsyncFileMeta,
    ) -> Result<(FileMeta, ImportFileMetaMethod), rusqlite::Error> {
        let exist = self.check_file_meta(post, &file_meta.filename)?;

        match exist {
            Some(id) => self.import_file_meta_by_update(author, post, id, file_meta),
            None => self.import_file_meta_by_create(author, post, file_meta),
        }
    }
    pub fn import_file_meta_by_create(
        &self,
        author: AuthorId,
        post: PostId,
        file_meta: UnsyncFileMeta,
    ) -> Result<(FileMeta, ImportFileMetaMethod), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT INTO file_metas (filename, author, post, mime, extra) VALUES (?, ?, ?, ?, ?) RETURNING id",
        )?;

        let filename = file_meta.filename;
        let mime = file_meta.mime;
        let extra = serde_json::to_string(&file_meta.extra).unwrap();
        let id: FileMetaId = stmt
            .query_row(params![&filename, &author, &post, &mime, &extra], |row| {
                row.get(0)
            })?;

        Ok((
            FileMeta {
                id,
                author,
                post,
                filename,
                mime,
                extra: file_meta.extra,
            },
            file_meta.method,
        ))
    }
    pub fn import_file_meta_by_update(
        &self,
        author: AuthorId,
        post: PostId,
        id: FileMetaId,
        file_meta: UnsyncFileMeta,
    ) -> Result<(FileMeta, ImportFileMetaMethod), rusqlite::Error> {
        // get filename and extra
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT filename, extra FROM file_metas WHERE id = ?")?;
        let (filename, extra) = stmt.query_row::<(String, String), _, _>(params![&id], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })?;

        // merge extra
        let mut extra: HashMap<String, String> = serde_json::from_str(&extra).unwrap();
        extra.extend(file_meta.extra.clone());

        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE file_metas SET extra = ? WHERE id = ?")?;
        stmt.execute(params![
            &serde_json::to_string(&file_meta.extra).unwrap(),
            &id
        ])?;

        Ok((
            FileMeta {
                id,
                author,
                post,
                filename,
                mime: file_meta.mime,
                extra,
            },
            file_meta.method,
        ))
    }
}

#[derive(Debug, Clone)]
pub struct UnsyncFileMeta {
    pub filename: String,
    pub mime: String,
    pub extra: HashMap<String, String>,
    pub method: ImportFileMetaMethod,
}

impl PartialEq for UnsyncFileMeta {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename
    }
}

impl Eq for UnsyncFileMeta {}

impl Hash for UnsyncFileMeta {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
    }
}

#[derive(Debug, Clone)]
pub enum ImportFileMetaMethod {
    Url(String),
    File(PathBuf),
    Data(Vec<u8>),
}
