use crate::{
    error::Result,
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
    /// Returns `Error` if there was an error accessing the database.
    pub fn import_platform(&self, platform: String) -> Result<PlatformId> {
        // find
        if let Some(id) = self.find_platform(&platform)? {
            return Ok(id);
        }

        // insert
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO platforms (name) VALUES (?) RETURNING id")?;
        Ok(stmt.query_row([&platform], |row| row.get(0))?)
    }
}
