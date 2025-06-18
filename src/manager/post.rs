use chrono::{DateTime, Utc};
use rusqlite::{params, OptionalExtension};

use crate::{
    manager::{PostArchiverConnection, PostArchiverManager},
    AuthorId, CollectionId, Comment, Content, FileMetaId, PlatformId, Post, PostId, TagId,
};

//=============================================================
// Querying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Retrieve all posts in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn list_posts(&self) -> Result<Vec<Post>, rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached("SELECT * FROM posts")?;
        let posts = stmt.query_map([], Post::from_row)?;
        posts.collect()
    }

    /// Find a post by its source.
    ///
    /// # errors
    ///
    /// Returns `rusqlite::error` if there was an error accessing the database.
    pub fn find_post(&self, source: &str) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id FROM posts WHERE source = ?")?;

        stmt.query_row(params![source], |row| row.get(0)).optional()
    }

    /// Find a post by its source.
    ///     
    /// If you want to check if the post exists in the archive, use [`find_post`](Self::find_post) instead.
    ///
    /// # errors
    ///
    /// Returns `rusqlite::error` if there was an error accessing the database.
    /// Check if a the post exists in the archive by their source and updated date.
    pub fn find_post_with_updated(
        &self,
        source: &str,
        updated: &DateTime<Utc>,
    ) -> Result<Option<PostId>, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT id, updated FROM posts WHERE source = ?")?;

        stmt.query_row::<(PostId, DateTime<Utc>), _, _>(params![source], |row| {
            Ok((row.get_unwrap(0), row.get_unwrap(1)))
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
    /// Retrieve a post by its ID.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The post ID does not exist
    /// * There was an error accessing the database
    pub fn get_post(&self, id: &PostId) -> Result<Post, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("SELECT * FROM posts WHERE id = ?")?;

        stmt.query_row([id], Post::from_row)
    }
}

//=============================================================
// Modifying
//=============================================================
impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Add a new post to the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if:
    /// * The post already exists with the same source
    /// * There was an error accessing the database
    pub fn add_post(
        &self,
        title: String,
        source: Option<String>,
        platform: Option<PlatformId>,
        published: Option<DateTime<Utc>>,
        updated: Option<DateTime<Utc>>,
    ) -> Result<PostId, rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached(
                "INSERT INTO posts (title, source, platform, published, updated) VALUES (?, ?, ?, ?, ?) RETURNING id",
            )?;

        stmt.query_row(
            params![title, source, platform, published, updated],
            |row| row.get(0),
        )
    }
    /// Remove a post from the archive.
    ///
    /// This operation will also remove all file metadata, author associations, tag associations, and collection associations for the post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_post(&self, post: PostId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM posts WHERE id = ?")?;
        stmt.execute([post])?;
        Ok(())
    }
    /// Associate one or more authors with a post.
    ///
    /// Creates author associations between a post and the provided author IDs.
    /// Duplicate associations are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn add_post_authors(
        &self,
        post: PostId,
        authors: &[AuthorId],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO author_posts (author, post) VALUES (?, ?)")?;
        for author in authors {
            stmt.execute(params![author, post])?;
        }
        Ok(())
    }
    /// Remove one or more authors from a post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_post_authors(
        &self,
        post: PostId,
        authors: &[AuthorId],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM author_posts WHERE post = ? AND author = ?")?;
        for author in authors {
            stmt.execute(params![post, author])?;
        }
        Ok(())
    }

    /// Associate one or more tags with a post.
    ///
    /// Creates tag associations between a post and the provided tag IDs.
    /// Duplicate associations are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn add_post_tags(&self, post: PostId, tags: &[TagId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")?;

        for tag in tags {
            stmt.execute(params![post, tag])?;
        }
        Ok(())
    }

    /// Remove one or more tags from a post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_post_tags(&self, post: PostId, tags: &[TagId]) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM post_tags WHERE post = ? AND tag = ?")?;
        for tag in tags {
            stmt.execute(params![post, tag])?;
        }
        Ok(())
    }
    /// Associate one or more tags with a post.
    ///
    /// Creates tag associations between a post and the provided tag IDs.
    /// Duplicate associations are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn add_post_collections(
        &self,
        post: PostId,
        collections: &[CollectionId],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT OR IGNORE INTO collection_posts (collection, post) VALUES (?, ?)",
        )?;

        for collection in collections {
            stmt.execute(params![collection, post])?;
        }
        Ok(())
    }

    /// Remove one or more collections from a post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_post_collections(
        &self,
        post: PostId,
        collections: &[CollectionId],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM collection_posts WHERE collection = ? AND post = ?")?;
        for collection in collections {
            stmt.execute(params![collection, post])?;
        }
        Ok(())
    }
    /// Set a post's source URL.
    ///
    /// Sets the source identifier for a post, or removes it by passing `None`.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_source(
        &self,
        post: PostId,
        source: Option<String>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET source = ? WHERE id = ?")?;
        stmt.execute(params![source, post])?;
        Ok(())
    }
    /// Set a post's platform.
    ///
    /// Associates a file metadata ID as the post's thumbnail, or removes it by passing `None`.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_platform(
        &self,
        post: PostId,
        platform: Option<PlatformId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET platform = ? WHERE id = ?")?;
        stmt.execute(params![platform, post])?;
        Ok(())
    }
    /// Set a post's title.
    ///
    /// Sets a new title for the specified post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_title(&self, post: PostId, title: String) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET title = ? WHERE id = ?")?;
        stmt.execute(params![title, post])?;
        Ok(())
    }

    /// Set a post's thumbnail.
    ///
    /// Associates a file metadata ID as the post's thumbnail, or removes it by passing `None`.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_thumb(
        &self,
        post: PostId,
        thumb: Option<FileMetaId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, post])?;
        Ok(())
    }

    /// Set a post's content.
    ///
    /// Replaces the entire content of a post with new text and file entries.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_content(
        &self,
        post: PostId,
        content: Vec<Content>,
    ) -> Result<(), rusqlite::Error> {
        let content = serde_json::to_string(&content).unwrap();

        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET content = ? WHERE id = ?")?;

        stmt.execute(params![content, post])?;
        Ok(())
    }
    /// Set a post's comments.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_comments(
        &self,
        post: PostId,
        comments: Vec<Comment>,
    ) -> Result<(), rusqlite::Error> {
        let comments = serde_json::to_string(&comments).unwrap();

        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET comments = ? WHERE id = ?")?;
        stmt.execute(params![comments, post])?;
        Ok(())
    }
    /// Set a post's published timestamp.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_published(
        &self,
        post: PostId,
        published: DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET published = ? WHERE id = ?")?;
        stmt.execute(params![published, post])?;
        Ok(())
    }
    ///Set a post's updated timestamp.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_updated(
        &self,
        post: PostId,
        updated: DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, post])?;
        Ok(())
    }
    /// Set a post's updated timestamp if current timestamp is more recent.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_updated_by_latest(
        &self,
        post: PostId,
        updated: DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ? AND updated < ?")?;
        stmt.execute(params![updated, post, updated])?;
        Ok(())
    }
}
