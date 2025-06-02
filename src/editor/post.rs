use chrono::{DateTime, Utc};
use rusqlite::params;

use crate::{
    manager::{collection::CollectionIdOrRaw, PostArchiverConnection, PostArchiverManager},
    utils::{
        collection::CollectionLike,
        tag::{PlatformTagIdOrRaw, PlatformTagLike, TagIdOrRaw, TagLike},
    },
    AuthorId, Collection, CollectionId, Comment, Content, FileMetaId, PlatformTag, PlatformTagId,
    PostId, Tag, TagId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Associate one or more authors with a post.
    ///
    /// Creates author associations between a post and the provided author IDs.
    /// Duplicate associations are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_authors(
        &self,
        post: PostId,
        authors: &[AuthorId],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO author_posts (author, post) VALUES (?, ?)")?;
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
    pub fn add_post_tags(
        &self,
        post: PostId,
        tags: &[impl TagLike],
    ) -> Result<(), rusqlite::Error> {
        let tags = tags
            .into_iter()
            .map(|tag| self.get_tag(tag))
            .filter_map(|result| result.transpose())
            .collect::<Result<Vec<Tag>, rusqlite::Error>>()?;

        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")?;

        for tag in tags {
            stmt.execute(params![post, tag.id])?;
        }
        Ok(())
    }

    pub fn remove_post_tags(
        &self,
        post: PostId,
        tags: &[impl TagLike],
    ) -> Result<(), rusqlite::Error> {
        let tags = tags
            .into_iter()
            .map(|tag| self.get_tag(tag))
            .filter_map(|result| result.transpose())
            .collect::<Result<Vec<Tag>, rusqlite::Error>>()?;

        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM post_tags WHERE post = ? AND tag = ?")?;
        for tag in tags {
            stmt.execute(params![post, tag.id])?;
        }
        Ok(())
    }

    /// Associate one or more platform tags with a post.
    ///
    /// Creates tag associations between a post and the provided tag IDs.
    /// Duplicate associations are silently ignored.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn add_post_platform_tags(
        &self,
        post: PostId,
        tags: &[PlatformTagIdOrRaw],
    ) -> Result<(), rusqlite::Error> {
        let tags = tags
            .into_iter()
            .map(|tag| self.get_platform_tag(tag))
            .filter_map(|result| result.transpose())
            .collect::<Result<Vec<PlatformTag>, rusqlite::Error>>()?;

        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_platform_tags (post, tag) VALUES (?, ?)")?;
        for tag in tags {
            stmt.execute(params![post, tag.id])?;
        }
        Ok(())
    }

    pub fn remove_post_platform_tags(
        &self,
        post: PostId,
        tags: &[PlatformTagIdOrRaw],
    ) -> Result<(), rusqlite::Error> {
        let tags = tags
            .into_iter()
            .map(|tag| self.get_platform_tag(tag))
            .filter_map(|result| result.transpose())
            .collect::<Result<Vec<PlatformTag>, rusqlite::Error>>()?;

        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM post_platform_tags WHERE post = ? AND tag = ?")?;
        for tag in tags {
            stmt.execute(params![post, tag.id])?;
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
        collections: &[impl CollectionLike],
    ) -> Result<(), rusqlite::Error> {
        let tags = collections
            .into_iter()
            .map(|collection| self.get_collection(collection))
            .filter_map(|result| result.transpose())
            .collect::<Result<Vec<Collection>, rusqlite::Error>>()?;

        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO post_tags (post, tag) VALUES (?, ?)")?;

        for tag in tags {
            stmt.execute(params![post, tag.id])?;
        }
        Ok(())
    }

    pub fn remove_post_collections(
        &self,
        post: PostId,
        collections: &[impl CollectionLike],
    ) -> Result<(), rusqlite::Error> {
        let tags = collections
            .into_iter()
            .map(|collection| self.get_collection(collection))
            .filter_map(|result| result.transpose())
            .collect::<Result<Vec<Collection>, rusqlite::Error>>()?;

        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM post_tags WHERE post = ? AND tag = ?")?;
        for tag in tags {
            stmt.execute(params![post, tag.id])?;
        }
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
        content: &Vec<Content>,
    ) -> Result<(), rusqlite::Error> {
        let content = serde_json::to_string(content).unwrap();

        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET content = ? WHERE id = ?")?;
        stmt.execute(params![content, post])?;
        Ok(())
    }
    /// Set or clear a post's source URL.
    ///
    /// Sets the source identifier for a post, or removes it by passing `None`.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_source(
        &self,
        post: PostId,
        source: &Option<String>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET source = ? WHERE id = ?")?;
        stmt.execute(params![source, post])?;
        Ok(())
    }
    /// Update a post's title.
    ///
    /// Sets a new title for the specified post.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_title(&self, post: PostId, title: &str) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET title = ? WHERE id = ?")?;
        stmt.execute(params![title, post])?;
        Ok(())
    }
    /// Set or remove a post's thumbnail image.
    ///
    /// Associates a file metadata ID as the post's thumbnail, or removes it by passing `None`.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_thumb(
        &self,
        post: PostId,
        thumb: &Option<FileMetaId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, post])?;
        Ok(())
    }
    /// Replace all comments on a post.
    ///
    /// Updates the post with a new set of comments, replacing any existing ones.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_comments(
        &self,
        post: PostId,
        comments: &Vec<Comment>,
    ) -> Result<(), rusqlite::Error> {
        let comments = serde_json::to_string(comments).unwrap();

        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET comments = ? WHERE id = ?")?;
        stmt.execute(params![comments, post])?;
        Ok(())
    }
    /// Update when a post was last modified.
    ///
    /// Sets the timestamp indicating when this post was last changed.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_updated(
        &self,
        post: PostId,
        updated: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, post])?;
        Ok(())
    }
    /// Update a post's timestamp if the new one is more recent.
    ///
    /// Only updates the last modified time if the new timestamp is more recent than the existing one.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_updated_by_latest(
        &self,
        post: PostId,
        updated: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET updated = ? WHERE id = ? AND updated < ?")?;
        stmt.execute(params![updated, post, updated])?;
        Ok(())
    }
    /// Set when a post was originally published.
    ///
    /// Updates the timestamp indicating when this post was first published.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn set_post_published(
        &self,
        post: PostId,
        published: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE posts SET published = ? WHERE id = ?")?;
        stmt.execute(params![published, post])?;
        Ok(())
    }
}
