//! FileMeta point-query helpers (no builder — use direct methods).

use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    utils::macros::AsTable,
    FileMeta, FileMetaId, PostId,
};

impl<C: PostArchiverConnection> PostArchiverManager<C> {
    /// Fetch a single [`FileMeta`] by primary key.
    pub fn get_file_meta(&self, id: FileMetaId) -> Result<Option<FileMeta>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM file_metas WHERE id = ?")?;
        stmt.query_row([id], FileMeta::from_row).optional()
    }

    /// Find a [`FileMetaId`] by owning `post` and `filename`.
    pub fn find_file_meta(
        &self,
        post: PostId,
        filename: &str,
    ) -> Result<Option<FileMetaId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM file_metas WHERE post = ? AND filename = ?")?;
        stmt.query_row(rusqlite::params![post, filename], |row| row.get(0))
            .optional()
    }
}
