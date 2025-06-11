use rusqlite::{params, OptionalExtension};

use crate::{Platform, PlatformId, Post, Tag};

use super::{PostArchiverConnection, PostArchiverManager};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all platforms in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_platforms(&self) -> Result<Vec<Platform>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM platforms")?;
        let platforms = stmt.query_map([], Platform::from_row)?;
        platforms.collect()
    }

    /// Find a platform by its name.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn find_platform(&self, name: &str) -> Result<Option<PlatformId>, rusqlite::Error> {
        if let Some(platform) = self.cache.platforms.get(name) {
            return Ok(Some(*platform));
        }

        let query = "SELECT id FROM platforms WHERE name = ?";
        let mut stmt = self.conn().prepare_cached(query)?;
        let id = stmt.query_row([name], |row| row.get(0)).optional();

        if let Ok(Some(id)) = id {
            self.cache.platforms.insert(name.to_string(), id);
        }

        id
    }
    /// Retrieve a platform by its ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The platform ID does not exist
    /// * There was an error accessing the database
    pub fn get_platform(&self, id: &PlatformId) -> Result<Platform, rusqlite::Error> {
        let query = "SELECT * FROM platforms WHERE id = ?";
        let mut stmt = self.conn().prepare_cached(query)?;
        stmt.query_row([id], Platform::from_row)
    }
}

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Add a new platform to the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The platform already exists
    /// * There was an error accessing the database
    pub fn add_platform(&self, platform: String) -> Result<PlatformId, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO platforms (name) VALUES (?) RETURNING id")?;
        let id = stmt.query_row([&platform], |row| row.get(0));

        if let Ok(id) = id {
            self.cache.platforms.insert(platform, id);
        }

        id
    }

    /// Remove a platform from the archive.
    ///
    /// This operation will also set the platform to UNKNOWN for all author aliases and posts with the platform.
    /// But it will delete tags associated with the platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_platform(&self, id: &PlatformId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM platforms WHERE id = ?")?;
        stmt.execute([id])?;
        Ok(())
    }

    /// Set the name of a platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
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
    /// List all tags associated with a platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_platform_tags(
        &self,
        platform: &Option<PlatformId>,
    ) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM tags WHERE platform = ?")?;
        let tags = stmt.query_map([platform], Tag::from_row)?;
        tags.collect()
    }
    /// List all posts associated with a platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_platform_posts(
        &self,
        platform: &Option<PlatformId>,
    ) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE platform = ?")?;
        let posts = stmt.query_map([platform], Post::from_row)?;
        posts.collect()
    }
}

impl Platform {
    /// Retrieve all tags associated with this platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn tags(&self, manager: &PostArchiverManager) -> Result<Vec<Tag>, rusqlite::Error> {
        manager.list_platform_tags(&Some(self.id))
    }
    /// Retrieve all posts associated with this platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn posts(&self, manager: &PostArchiverManager) -> Result<Vec<Post>, rusqlite::Error> {
        manager.list_platform_posts(&Some(self.id))
    }
}
