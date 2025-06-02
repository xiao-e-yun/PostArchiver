use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    Author, Post, PostId, Tag,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// List all posts in the archive.
    pub fn list_posts(&self) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM posts")?;
        let posts = stmt.query_map([], |row| Post::from_row(row))?;
        posts.collect()
    }

    /// Look up a post in the archive by its source identifier.
    ///
    /// Search for a post with the given source identifier, returning its ID if found
    /// or `None` if not found.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error querying the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     if let Some(id) = manager.check_post(&"https://example.com/post/123".to_string())? {
    ///         println!("Post exists with ID: {}", id);
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub fn check_post(&self, source: &str) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE source = ?")?;

        stmt.query_row(params![source], |row| row.get(0)).optional()
    }
    /// Check if a the post exists in the archive by their source and updated date.
    ///     
    /// This is useful to check if the post has been updated since the last time it was imported.
    /// If you want to check if the post exists in the archive, use [`check_post`](Self::check_post) instead.
    pub fn check_post_with_updated(
        &self,
        source: &str,
        updated: &DateTime<Utc>,
    ) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id, updated FROM posts WHERE source = ?")?;

        stmt.query_row::<(PostId, DateTime<Utc>), _, _>(params![source], |row| {
            Ok((row.get(0)?, row.get(1)?))
        })
        .optional()
        .map(|query| {
            query.and_then(|(id, last_update)| {
                if &last_update >= updated {
                    Some(id)
                } else {
                    None
                }
            })
        })
    }
    /// Retrieve an post's complete information from the archive.
    ///
    /// Fetches all information about a post including its source, author, and metadata.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The post ID does not exist
    /// * There was an error accessing the database
    /// * The stored data is malformed
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::PostId;
    /// fn example(manager: &PostArchiverManager, id: PostId) -> Result<(), Box<dyn std::error::Error>> {
    ///     let post = manager.get_post(&id)?;
    ///     println!("Post title: {}", post.title);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn get_post(&self, id: &PostId) -> Result<Post, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE id = ?")?;
        stmt.query_row([id], |row| Post::from_row(row))
    }
    /// Retrieve all authors associated with a post.
    ///
    /// Fetches all authors that have contributed to the given post ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::PostId;
    /// fn example(manager: &PostArchiverManager, post_id: PostId) -> Result<(), Box<dyn
    /// std::error::Error>> {
    ///     let authors = manager.get_post_authors(&post_id)?;
    ///     for author in authors {
    ///         println!("Post author: {}", author.name);
    ///     };
    ///     Ok(())
    /// }
    /// ```
    pub fn get_post_authors(&self, post: &Post) -> Result<Vec<Author>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT authors.* FROM authors INNER JOIN post_authors ON post_authors.author = authors.id WHERE post_authors.post = ?")?;
        let authors = stmt.query_map([post.id], |row| Author::from_row(row))?;
        authors.collect()
    }
}
