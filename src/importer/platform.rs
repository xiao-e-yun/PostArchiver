use crate::{
    manager::{
        platform::{PlatformIdOrRaw, PlatformLike},
        PostArchiverConnection, PostArchiverManager,
    },
    PlatformId,
};
use rusqlite::OptionalExtension;

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
    pub fn import_platform(
        &self,
        platform: impl PlatformLike,
    ) -> Result<PlatformId, rusqlite::Error> {
        let name = match platform.id() {
            Some(id) => return Ok(id),
            None => platform.raw().unwrap(),
        };
        // check cache
        if let Some(id) = self.cache.platforms.get(name) {
            return Ok(*id);
        }

        // check if tag exists
        let exist = self
            .conn()
            .query_row("SELECT id FROM platforms WHERE name = ?", [name], |row| {
                row.get(0)
            })
            .optional()?;

        let id = match exist {
            // if tag exists, return the id
            Some(id) => {
                self.cache.platforms.insert(name.to_string(), id);
                return Ok(id);
            }
            // if tag does not exist, insert the tag and return the id
            None => self.conn().query_row(
                "INSERT INTO platforms (name) VALUES (?) RETURNING id",
                [&name],
                |row| row.get(0),
            ),
        }?;

        // insert into cache
        self.cache.platforms.insert(name.to_string(), id);

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
    pub fn import_platforms(
        &self,
        platforms: Vec<impl PlatformLike>,
    ) -> Result<Vec<PlatformId>, rusqlite::Error> {
        platforms
            .into_iter()
            .map(|name| self.import_platform(name))
            .collect()
    }
}
