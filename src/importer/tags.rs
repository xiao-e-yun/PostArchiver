use rusqlite::OptionalExtension;

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    PostTagId,
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
    /// # use post_archiver::manager::PostArchiverManager;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let tag_id = manager.import_tag("rust")?;
    ///     println!("Imported tag with ID: {}", tag_id);
    ///     Ok(())
    /// }
    /// ```
    pub fn import_tag(&self, tag: &str) -> Result<PostTagId, rusqlite::Error> {
        // check cache
        if let Some(id) = self.cache.tags.lock().unwrap().get(tag) {
            return Ok(*id);
        }

        // check if tag exists
        let exist = self
            .conn()
            .query_row("SELECT id FROM tags WHERE name = ?", [&tag], |row| {
                row.get(0)
            })
            .optional()?;

        let id: PostTagId = match exist {
            // if tag exists, return the id
            Some(id) => {
                self.cache.tags.lock().unwrap().insert(tag.to_string(), id);
                return Ok(id);
            }
            // if tag does not exist, insert the tag and return the id
            None => self.conn().query_row(
                "INSERT INTO tags (name) VALUES (?) RETURNING id",
                [&tag],
                |row| row.get(0),
            ),
        }?;

        // insert into cache
        self.cache.tags.lock().unwrap().insert(tag.to_string(), id);

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
    /// # use post_archiver::manager::PostArchiverManager;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let tags = ["tutorial", "rust", "programming"];
    ///     
    ///     let tag_ids = manager.import_tags(&tags)?;
    ///     println!("Imported {} tags", tag_ids.len());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_tags<S>(&self, tags: &[S]) -> Result<Vec<PostTagId>, rusqlite::Error>
    where
        S: AsRef<str>,
    {
        tags.iter()
            .map(|tag| self.import_tag(tag.as_ref()))
            .collect()
    }
}
