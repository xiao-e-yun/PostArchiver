use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    PlatformId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Import a platform into the archive.
    ///
    /// If the platform already exists, it returns the existing ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn import_platform(&self, platform: String) -> Result<PlatformId, rusqlite::Error> {
        // find
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM platforms WHERE name = ?")?;
        if let Some(id) = stmt.query_row([&platform], |row| row.get(0)).optional()? {
            return Ok(id);
        }

        // insert
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO platforms (name) VALUES (?) RETURNING id")?;
        stmt.query_row([&platform], |row| row.get(0))
    }
}
