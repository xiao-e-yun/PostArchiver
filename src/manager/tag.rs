use rusqlite::{params, OptionalExtension};

use crate::{PlatformId, Post, PostId, Tag, TagId};

use super::{PostArchiverConnection, PostArchiverManager};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all tags in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_tags(&self) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM tags")?;
        let tags = stmt.query_map([], Tag::from_row)?;
        tags.collect()
    }

    /// Find a tag by its name and platform.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::PlatformId;
    /// fn example(manager: &PostArchiverManager) -> Result<(), rusqlite::Error> {
    ///     let first_tag = manager.find_tag(&"tag1")?;
    ///     let second_tag = manager.find_tag(&("tag2", PlatformId(2)))?;
    ///     let third_tag = manager.find_tag(&("tag3", Some(PlatformId(3))))?;
    ///     Ok(())
    /// }
    /// ```
    pub fn find_tag<U: FindTag>(&self, tag: &U) -> Result<Option<TagId>, rusqlite::Error> {
        let name = tag.name().to_string();
        let platform = tag.platform();

        let cache_key = (name.clone(), platform);
        if let Some(id) = self.cache.tags.get(&cache_key) {
            return Ok(Some(*id));
        }

        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM tags WHERE platform = ? AND name = ?")?;

        let id = stmt
            .query_row(params![name, platform], |row| row.get(0))
            .optional();

        if let Ok(Some(id)) = id {
            self.cache.tags.insert(cache_key, id);
        }

        id
    }

    /// Retrieve a tag by its ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The tag ID does not exist
    /// * There was an error accessing the database
    pub fn get_tag(&self, tag: &TagId) -> Result<Option<Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM tags WHERE id = ?")?;

        stmt.query_row([tag], Tag::from_row).optional()
    }
}

pub trait FindTag {
    fn name(&self) -> &str;
    fn platform(&self) -> Option<PlatformId> {
        None
    }
}

impl FindTag for &str {
    fn name(&self) -> &str {
        self
    }
}

impl FindTag for (&str, PlatformId) {
    fn name(&self) -> &str {
        self.0
    }
    fn platform(&self) -> Option<PlatformId> {
        Some(self.1)
    }
}

impl FindTag for (&str, Option<PlatformId>) {
    fn name(&self) -> &str {
        self.0
    }
    fn platform(&self) -> Option<PlatformId> {
        self.1
    }
}

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Add a new tag to the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The tag already exists
    /// * There was an error accessing the database
    pub fn add_tag(
        &self,
        name: String,
        platform: Option<PlatformId>,
    ) -> Result<TagId, rusqlite::Error> {
        let cache_key = (name.clone(), platform);
        if let Some(id) = self.cache.tags.get(&cache_key) {
            return Ok(*id);
        }

        let mut stmt = self
            .conn()
            .prepare_cached("INSERT INTO tags (name, platform) VALUES (?, ?) RETURNING id")?;

        let id = stmt.query_row(params![name, platform], |row| row.get(0))?;

        self.cache.tags.insert(cache_key, id);

        Ok(id)
    }

    /// Remove a tag from the archive.
    ///
    /// This will delete the posts associated with the tag as well, but will not delete the posts themselves.
    ///
    /// # Errors
    ///
    /// # Returns `rusqlite::Error` if:
    /// * The tag ID does not exist
    /// * There was an error accessing the database
    pub fn remove_tag(&self, tag: &TagId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM tags WHERE id = ?")?;

        stmt.execute([tag])?;
        Ok(())
    }

    /// Set the name of a tag.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_tag_name(&self, tag: &TagId, name: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE tags SET name = ? WHERE id = ? RETURNING platform")?;

        let platform: Option<PlatformId> = stmt.query_row(params![name, tag], |row| row.get(0))?;
        self.cache.tags.insert((name, platform), *tag);
        Ok(())
    }

    /// Set the platform of a tag.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_tag_platform(
        &self,
        tag: &TagId,
        platform: Option<PlatformId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE tags SET platform = ? WHERE id = ? RETURNING name")?;

        let name: String = stmt.query_row(params![platform, tag], |row| row.get(0))?;
        self.cache.tags.insert((name, platform), *tag);
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
    /// List all tags associated with a post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_post_tags(&self, post: &PostId) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT tags.* FROM tags INNER JOIN post_tags ON post_tags.tag = tags.id WHERE post_tags.post = ?")?;
        let tags = stmt.query_map([post], Tag::from_row)?;
        tags.collect()
    }

    /// List all posts associated with a tag.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_tag_posts(&self, tag: &TagId) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT posts.* FROM posts INNER JOIN post_tags ON post_tags.post = posts.id WHERE post_tags.tag = ?")?;
        let posts = stmt.query_map([tag], Post::from_row)?;
        posts.collect()
    }
}

impl Post {
    /// Retrieve all tags associated with this post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn tags(&self, manager: &PostArchiverManager) -> Result<Vec<Tag>, rusqlite::Error> {
        manager.list_post_tags(&self.id)
    }
}

impl Tag {
    /// Retrieve all posts associated with this tag.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn posts(&self, manager: &PostArchiverManager) -> Result<Vec<Post>, rusqlite::Error> {
        manager.list_tag_posts(&self.id)
    }
}
