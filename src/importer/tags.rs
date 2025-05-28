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
    /// # use post_archiver::{manager::{PostArchiverManager}, COLLECTION_CATEGORY};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let tag_id = manager.import_tag(COLLECTION_CATEGORY,"rust")?;
    ///     println!("Imported tag with ID: {}", tag_id);
    ///     Ok(())
    /// }
    /// ```
    pub fn import_tag(&self, category: &str, name: &str) -> Result<PostTagId, rusqlite::Error> {
        let cache_key = format!("{}:{}", category, name);
        // check cache
        if let Some(id) = self.cache.tags.lock().unwrap().get(&cache_key) {
            return Ok(*id);
        }

        // check if tag exists
        let exist = self
            .conn()
            .query_row(
                "SELECT id FROM tags WHERE category = ? AND name = ?",
                [category, name],
                |row| row.get(0),
            )
            .optional()?;

        let id: PostTagId = match exist {
            // if tag exists, return the id
            Some(id) => {
                self.cache.tags.lock().unwrap().insert(cache_key, id);
                return Ok(id);
            }
            // if tag does not exist, insert the tag and return the id
            None => self.conn().query_row(
                "INSERT INTO tags (category, name) VALUES (?, ?) RETURNING id",
                [category, name],
                |row| row.get(0),
            ),
        }?;

        // insert into cache
        self.cache.tags.lock().unwrap().insert(cache_key, id);

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
    ///     let tags = vec![
    ///         (COLLECTION_CATEGORY, "tag1"),
    ///         (COLLECTION_CATEGORY, "tag2"),
    ///         (COLLECTION_CATEGORY, "tag3"),
    ///     ];
    ///     
    ///     let tag_ids = manager.import_tags(&tags)?;
    ///     println!("Imported {} tags", tag_ids.len());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn import_tags<S>(&self, tags: &[(S, S)]) -> Result<Vec<PostTagId>, rusqlite::Error>
    where
        S: AsRef<str>,
    {
        tags.iter()
            .map(|(category, name)| self.import_tag(category.as_ref(), name.as_ref()))
            .collect()
    }
}
