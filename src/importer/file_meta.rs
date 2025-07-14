use std::{collections::HashMap, hash::Hash};

use serde_json::Value;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    FileMetaId, PostId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Create or update a file metadata entry in the archive.
    ///
    /// Takes a file metadata object and either creates a new entry or updates an existing one.
    /// if a file metadata with the same filename (and post id) already exists, it only updates metadata
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::importer::UnsyncFileMeta;
    /// # use post_archiver::PostId;
    /// # use std::collections::HashMap;
    /// fn example(manager: &PostArchiverManager, post_id: PostId) -> Result<(), rusqlite::Error> {
    ///     let file_meta = UnsyncFileMeta {
    ///         filename: "image.jpg".to_string(),
    ///         mime: "image/jpeg".to_string(),
    ///         extra: HashMap::new(),
    ///         data: (),
    ///     };
    ///     let meta = manager.import_file_meta(post_id, &file_meta)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_file_meta<U>(
        &self,
        post: PostId,
        file_meta: &UnsyncFileMeta<U>,
    ) -> Result<FileMetaId, rusqlite::Error> {
        match self.find_file_meta(post, &file_meta.filename)? {
            Some(id) => {
                // mime should not change
                self.set_file_meta_extra(id, file_meta.extra.clone())?;
                Ok(id)
            }
            None => self.add_file_meta(
                post,
                file_meta.filename.clone(),
                file_meta.mime.clone(),
                file_meta.extra.clone(),
            ),
        }
    }
}

/// Represents a file metadata that is not yet synced to the database.
#[derive(Debug, Clone)]
pub struct UnsyncFileMeta<T> {
    pub filename: String,
    pub mime: String,
    pub extra: HashMap<String, Value>,
    pub data: T,
}

impl<T> Hash for UnsyncFileMeta<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.filename.hash(state);
        self.mime.hash(state);
    }
}

impl<T> PartialEq for UnsyncFileMeta<T> {
    fn eq(&self, other: &Self) -> bool {
        self.filename == other.filename && self.mime == other.mime && self.extra == other.extra
    }
}

impl<T> Eq for UnsyncFileMeta<T> {}

impl<T> UnsyncFileMeta<T> {
    pub fn new(filename: String, mime: String, data: T) -> Self {
        Self {
            filename,
            mime,
            data,
            extra: HashMap::new(),
        }
    }

    pub fn extra(mut self, extra: HashMap<String, Value>) -> Self {
        self.extra = extra;
        self
    }
}
