use chrono::{DateTime, Utc};
use rusqlite::{params, ToSql};

use crate::{
    manager::{platform::PlatformIdOrRaw, PostArchiverConnection, PostArchiverManager},
    AuthorId, FileMetaId, PostId,
};

impl<T> PostArchiverManager<T>
where
    T: PostArchiverConnection,
{
    /// Set the author name by their id.
    pub fn set_author_name<S>(&self, author: &AuthorId, name: S) -> Result<(), rusqlite::Error>
    where
        S: AsRef<str> + ToSql,
    {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET name = ? WHERE id = ?")?;
        stmt.execute(params![name, &author])?;
        Ok(())
    }
    /// Set an author's last updated timestamp.
    ///
    /// Update the timestamp indicating when this author's information was last modified.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::AuthorId;
    /// # use chrono::Utc;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     let now = Utc::now();
    ///     
    ///     manager.set_author_updated(author_id, &now)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_updated(
        &self,
        author: AuthorId,
        updated: &DateTime<Utc>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = ? WHERE id = ?")?;
        stmt.execute(params![updated, &author])?;
        Ok(())
    }
    /// Set the author's last updated time to match their most recent post.
    ///
    /// Find the author's most recently updated post and use its timestamp as the
    /// author's last updated time.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::AuthorId;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     
    ///     manager.set_author_updated_by_latest(author_id)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_updated_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET updated = (SELECT posts.updated FROM author_posts JOIN posts ON author_posts.post = posts.id WHERE author_posts.author = ? ORDER BY posts.updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }
    /// Set or remove an author's thumbnail image.
    ///
    /// Associates a file metadata ID as the author's thumbnail image, or removes it
    /// by passing `None`. The specified file must already exist in the archive.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::{AuthorId, FileMetaId};
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     let thumb_file_id = FileMetaId(1);
    ///     
    ///     // Set a thumbnail
    ///     manager.set_author_thumb(author_id, Some(thumb_file_id))?;
    ///     
    ///     // Remove the thumbnail
    ///     manager.set_author_thumb(author_id, None)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_thumb(
        &self,
        author: AuthorId,
        thumb: Option<FileMetaId>,
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = ? WHERE id = ?")?;
        stmt.execute(params![thumb, &author])?;
        Ok(())
    }
    /// Set the author's thumbnail to their most recent post's thumbnail.
    ///
    /// Find the most recent post by this author that has a thumbnail and use it as
    /// the author's thumbnail image.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    ///
    /// # Examples
    ///
    /// ```
    /// # use post_archiver::manager::PostArchiverManager;
    /// # use post_archiver::AuthorId;
    /// fn example() -> Result<(), Box<dyn std::error::Error>> {
    ///     let manager = PostArchiverManager::open_in_memory()?;
    ///     let author_id = AuthorId(1);
    ///     
    ///     // Update author thumbnail from latest post
    ///     manager.set_author_thumb_by_latest(author_id)?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn set_author_thumb_by_latest(&self, author: AuthorId) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("UPDATE authors SET thumb = (SELECT posts.thumb FROM author_posts JOIN posts ON author_posts.post = posts.id WHERE author_posts.author = ? AND posts.thumb IS NOT NULL ORDER BY posts.updated DESC LIMIT 1) WHERE id = ?")?;
        stmt.execute(params![author, author])?;
        Ok(())
    }
    /// Add aliases for an author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn add_author_aliases(
        &self,
        author: &AuthorId,
        aliases: &[(PlatformIdOrRaw, String, Option<String>)],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self.conn().prepare_cached(
            "INSERT OR REPLACE INTO author_aliases (target, platform, source, link) VALUES (?, ?, ?, ?)",
        )?;
        for (platform, source, link) in aliases.iter() {
            let platform_id = match platform {
                PlatformIdOrRaw::Id(platform) => *platform,
                PlatformIdOrRaw::Raw(platform) => {
                    // Try to find existing platform first, create if not found
                    if let Some(existing) = self.get_platform(&platform.as_str())? {
                        existing.id
                    } else {
                        self.import_platform(platform.as_str())?
                    }
                }
            };
            stmt.execute(params![author, platform_id, source, link])?;
        }
        Ok(())
    }
    /// Remove aliases for an author.
    ///
    /// # Errors
    ///
    /// Returns `rusqlite::Error` if there was an error accessing the database.
    pub fn remove_author_aliases<S>(
        &self,
        author: &AuthorId,
        aliases: &[(PlatformIdOrRaw, S)],
    ) -> Result<(), rusqlite::Error>
    where
        S: AsRef<str>,
    {
        let mut stmt = self.conn().prepare_cached(
            "DELETE FROM author_aliases WHERE platform = ? AND source = ? AND target = ?",
        )?;
        for (platform, source) in aliases.iter() {
            let platform_id = match platform {
                PlatformIdOrRaw::Id(platform) => *platform,
                PlatformIdOrRaw::Raw(platform) => {
                    // Try to find existing platform, skip if not found
                    if let Some(existing) = self.get_platform(&platform.as_str())? {
                        existing.id
                    } else {
                        continue; // Skip if platform doesn't exist
                    }
                }
            };
            stmt.execute(params![platform_id, source.as_ref(), author])?;
        }
        Ok(())
    }
    pub fn add_author_posts(
        &self,
        author: &AuthorId,
        post: &[PostId],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("INSERT OR IGNORE INTO author_posts (author, post) VALUES (?, ?)")?;
        for &post in post.iter() {
            stmt.execute(params![author, post])?;
        }
        Ok(())
    }
    pub fn remove_author_posts(
        &self,
        author: &AuthorId,
        post: &[PostId],
    ) -> Result<(), rusqlite::Error> {
        let mut stmt = self
            .conn()
            .prepare_cached("DELETE FROM author_posts WHERE author = ? AND post = ?")?;
        for &post in post.iter() {
            stmt.execute(params![author, post])?;
        }
        Ok(())
    }
}
