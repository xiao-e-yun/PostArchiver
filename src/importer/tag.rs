use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{platform::PlatformIdOrRaw, PostArchiverConnection, PostArchiverManager},
    utils::tag::{PlatformTagIdOrRaw, PlatformTagLike, TagIdOrRaw, TagLike},
    PlatformTagId, TagId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Import a single tag into the archive.
    ///
    /// Looks up the tag by name and returns its ID if it exists, otherwise creates
    /// a new tag entry. Uses an in-memory cache to speed up repeated lookups.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::{manager::{PostArchiverManager}, COLLECTION_CATEGORY};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let tag_id = manager.import_tag("rust")?;
    ///     println!("Imported tag with ID: {}", tag_id);
    ///     Ok(())
    /// }
    /// ```
    pub fn import_tag(&self, name: impl TagLike) -> Result<TagId, rusqlite::Error> {
        let name = match name.id() {
            Some(id) => return Ok(id),
            None => name.raw().unwrap().to_string(),
        };

        // check if tag exists
        let exist = self
            .conn()
            .query_row("SELECT id FROM tags WHERE name = ?", [&name], |row| {
                row.get(0)
            })
            .optional()?;

        let id = match exist {
            // if tag exists, return the id
            Some(id) => {
                self.cache.tags.insert(name.to_string(), id);
                return Ok(id);
            }
            // if tag does not exist, insert the tag and return the id
            None => self.conn().query_row(
                "INSERT INTO tags (name) VALUES (?) RETURNING id",
                [&name],
                |row| row.get(0),
            ),
        }?;

        // insert into cache
        self.cache.tags.insert(name, id);

        Ok(id)
    }

    /// Import multiple tags into the archive at once.
    ///
    /// Takes a slice of tag names and imports each one, returning their IDs in the same order.
    /// Creates new tags for ones that don't exist, or returns existing tag IDs.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::{manager::PostArchiverManager, COLLECTION_CATEGORY};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let tags = vec![ "tag1", "tag2", "tag3" ];
    ///     
    ///     let tag_ids = manager.import_tags(&tags)?;
    ///     println!("Imported {} tags", tag_ids.len());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_tags(&self, tags: Vec<impl TagLike>) -> Result<Vec<TagId>, rusqlite::Error> {
        tags.into_iter().map(|name| self.import_tag(name)).collect()
    }
    /// Import a single tag into the archive.
    ///
    /// Looks up the tag by name and returns its ID if it exists, otherwise creates
    /// a new tag entry. Uses an in-memory cache to speed up repeated lookups.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::{manager::{PostArchiverManager}, COLLECTION_CATEGORY};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let tag_id = manager.import_tag("rust")?;
    ///     println!("Imported tag with ID: {}", tag_id);
    ///     Ok(())
    /// }
    /// ```
    pub fn import_platform_tag(
        &self,
        tag: impl PlatformTagLike,
    ) -> Result<PlatformTagId, rusqlite::Error> {
        let (platform, name) = match tag.id() {
            Some(id) => return Ok(id),
            None => tag.raw().unwrap(),
        };
        let (platform, name) = (platform.clone(), name.to_string());

        let platform = self.import_platform(platform)?;
        let cache_key = (platform, name.clone());

        // check cache
        if let Some(id) = self.cache.platform_tags.get(&cache_key) {
            return Ok(*id);
        }

        // check if tag exists
        let exist = self
            .conn()
            .query_row(
                "SELECT id FROM platform_tags WHERE platform = ? AND name = ?",
                params![platform, &name],
                |row| row.get(0),
            )
            .optional()?;

        let id = match exist {
            // if tag exists, return the id
            Some(id) => {
                self.cache.platform_tags.insert(cache_key.clone(), id);
                return Ok(id);
            }
            // if tag does not exist, insert the tag and return the id
            None => self.conn().query_row(
                "INSERT INTO platform_tags (platform, name) VALUES (?,?) RETURNING id",
                params![platform, &name],
                |row| row.get(0),
            ),
        }?;

        // insert into cache
        self.cache.platform_tags.insert(cache_key, id);

        Ok(id)
    }

    /// Import multiple tags into the archive at once.
    ///
    /// Takes a slice of tag names and imports each one, returning their IDs in the same order.
    /// Creates new tags for ones that don't exist, or returns existing tag IDs.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::{manager::PostArchiverManager, COLLECTION_CATEGORY};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let tags = vec![ "tag1", "tag2", "tag3" ];
    ///     
    ///     let tag_ids = manager.import_platform_tags(&tags)?;
    ///     println!("Imported {} tags", tag_ids.len());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_platform_tags(
        &self,
        tags: Vec<impl PlatformTagLike>,
    ) -> Result<Vec<PlatformTagId>, rusqlite::Error> {
        tags.into_iter()
            .map(|tag| self.import_platform_tag(tag))
            .collect()
    }
}
