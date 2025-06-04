use rusqlite::{params, OptionalExtension};

use crate::{Platform, PlatformId};

use super::{PostArchiverConnection, PostArchiverManager};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all platforms.
    pub fn list_platforms(&self) -> Result<Vec<Platform>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM platforms")?;
        let platforms = stmt.query_map([], |row| Platform::from_row(row))?;
        platforms.collect()
    }

    pub fn find_platform(&self, name: &str) -> Result<Option<PlatformId>, rusqlite::Error> {
        if let Some(platform) = self.cache.platforms.get(name) {
            return Ok(Some(platform.clone()));
        }

        let query = "SELECT id FROM platforms WHERE name = ?";
        let mut stmt = self.conn().prepare_cached(query)?;
        let id = stmt.query_row([name], |row| row.get(0)).optional();

        if let Ok(Some(id)) = id {
            self.cache.platforms.insert(name.to_string(), id);
        }

        id
    }
    /// Get a platform by its id or name.
    pub fn get_platform(&self, id: &PlatformId) -> Result<Option<Platform>, rusqlite::Error> {
        let query = "SELECT * FROM platforms WHERE id = ?";
        let mut stmt = self.conn().prepare_cached(query)?;
        stmt.query_row([id], |row| Platform::from_row(row))
            .optional()
    }
}

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn add_platform(&self, platform: String) -> Result<PlatformId, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO platforms name VALUES ? RETURNING id")?;
        let id = stmt.query_row([&platform], |row| row.get(0));

        if let Ok(id) = id {
            self.cache.platforms.insert(platform, id);
        }

        id
    }

    pub fn remove_platform(&self, id: &PlatformId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM platforms WHERE id = ?")?;
        stmt.execute([id])?;
        Ok(())
    }

    pub fn set_platform_name(&self, id: &PlatformId, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE platforms SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, id])?;
        self.cache.platforms.insert(name.clone(), *id);
        Ok(())
    }
}

//=============================================================
// Relationships
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    pub fn list_platform_tags(
        &self,
        platform: &Option<PlatformId>,
    ) -> Result<Vec<crate::Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM tags WHERE platform = ?")?;
        let tags = stmt.query_map([platform], |row| crate::Tag::from_row(row))?;
        tags.collect()
    }
    pub fn list_platform_posts(
        &self,
        platform: &Option<PlatformId>,
    ) -> Result<Vec<crate::Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE platform = ?")?;
        let posts = stmt.query_map([platform], |row| crate::Post::from_row(row))?;
        posts.collect()
    }
}

impl Platform {
    /// Retrieve all tags associated with this platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn tags(&self, manager: &PostArchiverManager) -> Result<Vec<crate::Tag>, rusqlite::Error> {
        manager.list_platform_tags(&Some(self.id))
    }
    /// Retrieve all posts associated with this platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn posts(
        &self,
        manager: &PostArchiverManager,
    ) -> Result<Vec<crate::Post>, rusqlite::Error> {
        manager.list_platform_posts(&Some(self.id))
    }
}
