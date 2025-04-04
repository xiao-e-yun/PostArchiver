use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    Comment, Content, FileMeta, Post, PostId, PostTag, Tag,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
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
        stmt.query_row([id], |row| {
            let content: String = row.get("content")?;
            let content: Vec<Content> = serde_json::from_str(&content).unwrap();
            let comments: String = row.get("comments")?;
            let comments: Vec<Comment> = serde_json::from_str(&comments).unwrap();
            Ok(Post {
                id: row.get("id")?,
                author: row.get("author")?,
                source: row.get("source")?,
                title: row.get("title")?,
                content,
                thumb: row.get("thumb")?,
                comments,
                updated: row.get("updated")?,
                published: row.get("published")?,
            })
        })
    }
    /// Retrieve all tags associated with a post.
    pub fn get_post_tags(&self, id: &PostId) -> Result<Vec<Tag>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT tags.* FROM tags INNER JOIN post_tags ON post_tags.tag = tags.id WHERE post_tags.post = ?")?;
        let tags = stmt.query_map([id], |row| {
            Ok(Tag {
                id: row.get("id")?,
                name: row.get("name")?,
            })
        })?;
        tags.collect()
    }
}

/// Trait to get post
pub trait GetPost {
    /// Get post id
    fn post_id(&self) -> &PostId;
    /// Get post
    fn post(&self, manager: &PostArchiverManager) -> Result<Post, rusqlite::Error> {
        manager.get_post(self.post_id())
    }
    /// Get post's tags
    fn post_tags(&self, manager: &PostArchiverManager) -> Result<Vec<Tag>, rusqlite::Error> {
        manager.get_post_tags(self.post_id())
    }
}

impl GetPost for PostId {
    fn post_id(&self) -> &PostId {
        self
    }
}

impl GetPost for Post {
    fn post_id(&self) -> &PostId {
        &self.id
    }
}

impl GetPost for FileMeta {
    fn post_id(&self) -> &PostId {
        &self.post
    }
}

impl GetPost for PostTag {
    fn post_id(&self) -> &PostId {
        &self.post
    }
}
