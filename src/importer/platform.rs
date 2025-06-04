use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    PlatformId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn import_platform(&self, platform: String) -> Result<PlatformId, rusqlite::Error> {
        match self.find_platform(&platform)? {
            Some(id) => Ok(id),
            None => self.add_platform(platform),
        }
    }
}
