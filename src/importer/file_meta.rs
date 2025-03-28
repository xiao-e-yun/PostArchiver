use std::{collections::HashMap, fmt::Display, hash::Hash, path::PathBuf};

use rusqlite::{params, OptionalExtension};
use serde_json::Value;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, FileMeta, FileMetaId, PostId,
};

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
    /// Create a new file metadata entry in the archive.
    ///
    /// Creates a new file metadata record for a given post, returning both the created
    /// metadata and the method for importing the actual file. The file itself must be
    /// archived separately after the metadata is created.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::importer::{UnsyncFileMeta, ImportFileMetaMethod};
    /// # use post_archiver::{AuthorId, PostId};
    /// # use std::collections::HashMap;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     let post_id = PostId(1);
    ///     
    ///     let file_meta = UnsyncFileMeta {
    ///         filename: "image.jpg".to_string(),
    ///         mime: "image/jpeg".to_string(),
    ///         extra: HashMap::new(),
    ///         method: ImportFileMetaMethod::None,
    ///     };
    ///     let (meta, method) = manager.import_file_meta(author_id, post_id, file_meta)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_file_meta(
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
        let extra = serde_json::to_string(&file_meta.extra).unwrap_or_default();
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
    /// Update an existing file metadata entry while preserving and merging its extra data.
    ///
    /// Updates the file metadata with new information and merges any extra data with
    /// existing values. Returns both the updated metadata and the method for importing
    /// the actual file.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Panics
    ///
    /// * When the file metadata ID does not exist
    /// * When the author does not exist in the archive
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::importer::{UnsyncFileMeta, ImportFileMetaMethod};
    /// # use post_archiver::{AuthorId, PostId, FileMetaId};
    /// # use std::collections::HashMap;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     let post_id = PostId(1);
    ///     let file_id = FileMetaId(1);
    ///     
    ///     let new_meta = UnsyncFileMeta {
    ///         filename: "updated.jpg".to_string(),
    ///         mime: "image/jpeg".to_string(),
    ///         extra: HashMap::new(),
    ///         method: ImportFileMetaMethod::None,
    ///     };
    ///     let (meta, method) = manager.import_file_meta_by_update(
    ///         author_id,
    ///         post_id,
    ///         file_id,
    ///         new_meta
    ///     )?;
    ///     
    ///     Ok(())
    /// }
    /// ```
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
        let mut extra: HashMap<String, Value> = serde_json::from_str(&extra).unwrap_or_default();
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

/// Represents a file metadata that is not yet synced to the database.
#[derive(Debug, Clone)]
pub struct UnsyncFileMeta {
    pub filename: String,
    pub mime: String,
    pub extra: HashMap<String, Value>,
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

/// Represents a method to import file metadata.
#[derive(Debug, Clone)]
pub enum ImportFileMetaMethod {
    /// The file is imported from a URL.
    Url(String),
    /// The file is imported from a local file.
    File(PathBuf),
    /// The file is imported from raw data.
    Data(Vec<u8>),
    /// The file is imported as phantom data.
    None,
}

impl Display for ImportFileMetaMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImportFileMetaMethod::Url(url) => write!(f, "Url({})", url),
            ImportFileMetaMethod::File(path) => write!(f, "File({})", path.display()),
            ImportFileMetaMethod::Data(data) => write!(f, "Data({} bytes)", data.len()),
            ImportFileMetaMethod::None => write!(f, "None"),
        }
    }
}
