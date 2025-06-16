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
        match self.find_platform(&platform)? {
            Some(id) => Ok(id),
            None => self.add_platform(platform),
        }
    }
}
